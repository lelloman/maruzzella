#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabHost {
    NavigationList,
    IdentityList,
    InspectorDetails,
    CommandList,
    TextBuffer,
}

#[derive(Clone, Debug)]
pub struct DockTabSpec {
    pub id: String,
    pub dock_id: String,
    pub title: String,
    pub tab_type: String,
    pub instance_key: Option<String>,
    pub host: TabHost,
    pub placeholder: String,
    pub closable: bool,
    pub close_prompt: Option<String>,
}

pub fn text_tab(id: &str, dock_id: &str, title: &str, body: &str, closable: bool) -> DockTabSpec {
    DockTabSpec {
        id: id.to_string(),
        dock_id: dock_id.to_string(),
        title: title.to_string(),
        tab_type: "text".to_string(),
        instance_key: None,
        host: TabHost::TextBuffer,
        placeholder: body.to_string(),
        closable,
        close_prompt: None,
    }
}
