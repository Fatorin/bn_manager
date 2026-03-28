use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;

use crate::database::mysql_pool;
use crate::model::game::Game;
use crate::model::match_history::{MatchHistory, Servant, Team};
use crate::model::pagination::{paginate, PaginationResult};

#[derive(Deserialize)]
pub struct MatchHistoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct PlayerInfo {
    username: String,
    #[allow(dead_code)]
    pid: i32,
    servant: Option<String>,
    kills: Option<i32>,
    deaths: Option<i32>,
    assists: Option<i32>,
    level: Option<i32>,
}

pub async fn get_match_histories(
    Query(params): Query<MatchHistoryQuery>,
) -> impl IntoResponse {
    let mut limit = params.limit.unwrap_or(10);
    if limit <= 0 {
        limit = 1;
    }
    if limit > 10 {
        limit = 10;
    }

    let mut offset = params.offset.unwrap_or(0);
    if offset < 0 {
        offset = 0;
    }

    let pool = mysql_pool();

    // Fetch games
    let games = match sqlx::query_as::<_, Game>(
        "SELECT id, map, datetime, duration FROM games ORDER BY datetime DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    {
        Ok(g) => g,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let mut match_histories = Vec::new();

    for game in &games {
        // Fetch player info with LEFT JOINs
        let player_infos = match sqlx::query_as::<_, PlayerInfo>(
            r#"
            SELECT
                wp.name AS username,
                wp.pid,
                v_servant.value_string AS servant,
                v_kills.value_int AS kills,
                v_deaths.value_int AS deaths,
                v_assists.value_int AS assists,
                v_level.value_int AS level
            FROM w3mmdplayers wp
            LEFT JOIN w3mmdvars v_servant ON v_servant.gameid = wp.gameid AND v_servant.pid = wp.pid AND v_servant.varname = 'servant'
            LEFT JOIN w3mmdvars v_kills   ON v_kills.gameid   = wp.gameid AND v_kills.pid   = wp.pid AND v_kills.varname   = 'kills'
            LEFT JOIN w3mmdvars v_deaths  ON v_deaths.gameid  = wp.gameid AND v_deaths.pid  = wp.pid AND v_deaths.varname  = 'deaths'
            LEFT JOIN w3mmdvars v_assists ON v_assists.gameid = wp.gameid AND v_assists.pid = wp.pid AND v_assists.varname = 'assists'
            LEFT JOIN w3mmdvars v_level   ON v_level.gameid   = wp.gameid AND v_level.pid   = wp.pid AND v_level.varname   = 'level'
            WHERE wp.gameid = ?
            "#,
        )
        .bind(game.id)
        .fetch_all(pool)
        .await
        {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        };

        // Fetch team info
        let team_infos = match sqlx::query_scalar::<_, Option<String>>(
            "SELECT value_string FROM w3mmdvars WHERE gameid = ? AND varname = 'team_info'",
        )
        .bind(game.id)
        .fetch_all(pool)
        .await
        {
            Ok(t) => t,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        };

        let teams = analyse(&team_infos, &player_infos);

        match_histories.push(MatchHistory {
            id: game.id,
            map: game.map.clone(),
            datetime: game.datetime,
            duration: game.duration,
            teams,
        });
    }

    // Total count
    let total = match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM games")
        .fetch_one(pool)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let (pages, current_page, has_next) = paginate(total, limit, offset);

    Json(PaginationResult {
        total,
        limit,
        offset,
        page: current_page,
        pages,
        has_next,
        data: match_histories,
    })
    .into_response()
}

fn analyse(team_infos: &[Option<String>], player_infos: &[PlayerInfo]) -> Vec<Team> {
    let mut team_map: HashMap<i32, Team> = HashMap::new();

    for team_info in team_infos {
        let (index, team_name, score) = match team_info_split(team_info) {
            Some(v) => v,
            None => continue,
        };

        if team_map.contains_key(&index) {
            tracing::warn!("team index duplicate: {}", index);
            return Vec::new();
        }

        team_map.insert(
            index,
            Team {
                index,
                name: team_name,
                score,
                servants: Vec::new(),
            },
        );
    }

    for player in player_infos {
        let servant_str = match &player.servant {
            Some(s) => s,
            None => return Vec::new(),
        };

        let (team_index, servant_name) = match player_info_split(servant_str) {
            Some(v) => v,
            None => continue,
        };

        let team = match team_map.get_mut(&team_index) {
            Some(t) => t,
            None => {
                tracing::warn!("not found team index: {}", team_index);
                continue;
            }
        };

        team.servants.push(Servant {
            user_name: player.username.clone(),
            name: servant_name,
            level: convert_point_int(player.level),
            kills: convert_point_int(player.kills),
            deaths: convert_point_int(player.deaths),
            assists: convert_point_int(player.assists),
        });
    }

    team_map.into_values().collect()
}

fn team_info_split(team_info: &Option<String>) -> Option<(i32, String, i32)> {
    let s = team_info.as_ref()?;
    let cleaned = s.trim_matches('"');
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() != 3 {
        tracing::warn!("team info format invalid: {}", s);
        return None;
    }
    let index = parts[0].parse::<i32>().ok()?;
    let score = parts[2].parse::<i32>().ok()?;
    Some((index, parts[1].to_string(), score))
}

fn player_info_split(servant: &str) -> Option<(i32, String)> {
    let cleaned = servant.trim_matches('"');
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() != 2 {
        tracing::warn!("servant info format invalid: {}", servant);
        return None;
    }
    let team_id = parts[0].parse::<i32>().ok()?;
    Some((team_id, parts[1].to_string()))
}

fn convert_point_int(value: Option<i32>) -> i32 {
    value.unwrap_or(-1)
}
