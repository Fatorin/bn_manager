-- MMR System Tables
-- Run this script against the MySQL ghost database

-- Replace game_bonus_processed with game_mmr_processed
CREATE TABLE IF NOT EXISTS game_mmr_processed (
    id INT AUTO_INCREMENT PRIMARY KEY,
    gameid INT NOT NULL UNIQUE,
    processed_at DATETIME NOT NULL
);

-- Score change audit log
CREATE TABLE IF NOT EXISTS score_change_logs (
    id INT AUTO_INCREMENT PRIMARY KEY,
    gameid INT NOT NULL,
    category VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    server VARCHAR(255) NOT NULL,
    mmr_before DOUBLE NOT NULL,
    mmr_after DOUBLE NOT NULL,
    mmr_delta DOUBLE NOT NULL,
    result_flag VARCHAR(10) NOT NULL,
    team_avg_mmr DOUBLE NOT NULL,
    opponent_avg_mmr DOUBLE NOT NULL,
    created_at DATETIME NOT NULL,
    INDEX idx_gameid (gameid),
    INDEX idx_name_category (name, category)
);