use crate::database::StoredConfig;
use crate::models::PlaybackSettings;

pub fn playback_settings_from_stored_config(config: &StoredConfig) -> PlaybackSettings {
    PlaybackSettings {
        auto_play: config.auto_play,
        auto_skip: config.auto_skip,
        skip_threshold: config.skip_threshold,
    }
}

pub fn stored_config_with_playback_settings(
    config: &StoredConfig,
    playback_settings: &PlaybackSettings,
) -> StoredConfig {
    StoredConfig {
        query: config.query.clone(),
        max_age_days: config.max_age_days,
        targets: config.targets.clone(),
        category_filter: config.category_filter.clone(),
        auto_play: playback_settings.auto_play,
        auto_skip: playback_settings.auto_skip,
        skip_threshold: playback_settings.skip_threshold,
    }
}

#[cfg(test)]
mod tests {
    use super::{playback_settings_from_stored_config, stored_config_with_playback_settings};
    use crate::database::StoredConfig;
    use crate::models::PlaybackSettings;

    #[test]
    fn reads_playback_settings_from_stored_config() {
        let config = StoredConfig {
            query: "VOCALOID".to_string(),
            max_age_days: Some(365),
            targets: "tags".to_string(),
            category_filter: Some("MUSIC".to_string()),
            auto_play: false,
            auto_skip: true,
            skip_threshold: 45,
        };

        let settings = playback_settings_from_stored_config(&config);

        assert_eq!(settings.auto_play, false);
        assert_eq!(settings.auto_skip, true);
        assert_eq!(settings.skip_threshold, 45);
    }

    #[test]
    fn writes_playback_settings_without_overwriting_scraper_config_fields() {
        let config = StoredConfig {
            query: "Miku".to_string(),
            max_age_days: Some(30),
            targets: "title".to_string(),
            category_filter: None,
            auto_play: true,
            auto_skip: false,
            skip_threshold: 30,
        };
        let playback_settings = PlaybackSettings {
            auto_play: false,
            auto_skip: true,
            skip_threshold: 60,
        };

        let next = stored_config_with_playback_settings(&config, &playback_settings);

        assert_eq!(next.query, "Miku");
        assert_eq!(next.max_age_days, Some(30));
        assert_eq!(next.targets, "title");
        assert_eq!(next.category_filter, None);
        assert_eq!(next.auto_play, false);
        assert_eq!(next.auto_skip, true);
        assert_eq!(next.skip_threshold, 60);
    }
}
