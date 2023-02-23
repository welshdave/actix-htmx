pub(crate) struct RequestHeaders;
pub(crate) struct ResponseHeaders;

impl RequestHeaders {
    pub(crate) const HX_REQUEST: &'static str = "hx-request";
    pub(crate) const HX_BOOSTED: &'static str = "hx-boosted";
    pub(crate) const HX_CURRENT_URL: &'static str = "hx-current-url";
    pub(crate) const HX_HISTORY_RESTORE_REQUEST: &'static str = "hx-history-restore-request";
    pub(crate) const HX_PROMPT: &'static str = "hx-prompt";
    pub(crate) const HX_TARGET: &'static str = "hx-target";
    pub(crate) const HX_TRIGGER: &'static str = "hx-trigger";
    pub(crate) const HX_TRIGGER_NAME: &'static str = "hx-trigger-name";
}

impl ResponseHeaders {
    pub(crate) const HX_PUSH_URL: &'static str = "hx-push-url";
    pub(crate) const HX_LOCATION: &'static str = "hx-location";
    pub(crate) const HX_REDIRECT: &'static str = "hx-redirect";
    pub(crate) const HX_REFRESH: &'static str = "hx-refresh";
    pub(crate) const HX_TRIGGER: &'static str = "hx-trigger";
    pub(crate) const HX_TRIGGER_AFTER_SETTLE: &'static str = "hx-trigger-after-settle";
    pub(crate) const HX_TRIGGER_AFTER_SWAP: &'static str = "hx-trigger-after-swap";
    pub(crate) const HX_RESWAP: &'static str = "hx-reswap";
    pub(crate) const HX_RETARGET: &'static str = "hx-retarget";
    pub(crate) const HX_RESELECT: &'static str = "hx-reselect";
    pub(crate) const HX_REPLACE_URL: &'static str = "hx-replace-url";
}
