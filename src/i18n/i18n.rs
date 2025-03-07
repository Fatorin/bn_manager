use crate::i18n::LANG_EN_US;
use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub static I18N: Lazy<I18n> = Lazy::new(|| {
    let mut i18n = I18n::new(LANG_EN_US);

    let i18n_path = Path::new("./i18n");
    if let Err(err) = i18n.load_resources_from_dir(i18n_path) {
        eprintln!("無法讀取翻譯文件：{}", err);
    }

    i18n
});

pub struct I18n {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    default_locale: String,
}

impl I18n {
    pub fn new(default_locale: &str) -> Self {
        I18n {
            bundles: HashMap::new(),
            default_locale: default_locale.to_string(),
        }
    }

    pub fn get(&self, key: &str, locale: &str) -> String {
        self.get_locale_text(key, locale, None)
    }

    pub fn get_with_arg(&self, key: &str, locale: &str, arg_name: &str, arg_value: &str) -> String {
        let mut args = FluentArgs::new();
        args.set(arg_name, FluentValue::from(arg_value));
        self.get_locale_text(key, locale, Some(&args))
    }

    pub fn get_with_args(&self, key: &str, locale: &str, args_vec: &[(&str, &str)]) -> String {
        let mut args = FluentArgs::new();
        for (name, value) in args_vec {
            args.set(*name, FluentValue::from(*value));
        }
        self.get_locale_text(key, locale, Some(&args))
    }

    fn add_resource_from_string(&mut self, locale: &str, source: &str) -> Result<(), String> {
        let resource = FluentResource::try_new(source.to_string()).unwrap_or_else(|(res, _)| res);

        let lang_id = match locale.parse() {
            Ok(id) => id,
            Err(_) => return Err(format!("無效的語言識別碼: {}", locale)),
        };

        let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
        bundle.set_use_isolating(false); // 不使用隔離字符

        match bundle.add_resource(resource) {
            Ok(_) => {
                self.bundles.insert(locale.to_string(), bundle);
                Ok(())
            }
            Err(errors) => Err(format!("添加資源時發生錯誤: {:?}", errors)),
        }
    }

    fn load_resources_from_dir(&mut self, dir: &Path) -> Result<(), String> {
        if !dir.is_dir() {
            return Err(format!("{} 不是有效的目錄", dir.display()));
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(err) => return Err(format!("無法讀取目錄 {}: {}", dir.display(), err)),
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => return Err(format!("無法讀取目錄項: {}", err)),
            };

            let path = entry.path();

            // 只處理 .ftl 文件
            if path.is_file() && path.extension().map_or(false, |ext| ext == "ftl") {
                // 從文件名獲取語言代碼 (不包括副檔名 .ftl)
                if let Some(file_stem) = path.file_stem() {
                    let locale = file_stem.to_string_lossy().to_string();

                    let content = match fs::read_to_string(&path) {
                        Ok(content) => content,
                        Err(err) => {
                            return Err(format!("無法讀取文件 {}: {}", path.display(), err))
                        }
                    };

                    if let Err(err) = self.add_resource_from_string(&locale, &content) {
                        return Err(format!("無法添加資源 {}: {}", path.display(), err));
                    }
                }
            }
        }

        Ok(())
    }

    fn get_locale_text(&self, key: &str, locale: &str, args: Option<&FluentArgs>) -> String {
        let bundle = self
            .bundles
            .get(locale)
            .or_else(|| self.bundles.get(&self.default_locale));

        if let Some(bundle) = bundle {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errors = vec![];
                    let result = bundle.format_pattern(pattern, args, &mut errors);
                    return result.to_string();
                }
            }
        }

        key.to_string()
    }
}
