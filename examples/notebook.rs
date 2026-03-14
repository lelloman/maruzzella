use maruzzella::{
    default_product_spec, run, text_tab, MaruzzellaConfig, SplitAxis, TabGroupSpec,
    WorkbenchNodeSpec,
};

fn main() {
    let mut product = default_product_spec();
    product.branding.title = "Notebook".to_string();
    product.branding.search_placeholder = "Search notes, boards, and logs".to_string();
    product.branding.status_text = "Example app built on Maruzzella".to_string();

    product.menu_roots[0].label = "Notebook".to_string();
    product.menu_items[1].label = "About Notebook".to_string();
    product.commands[2].title = "About Notebook".to_string();

    product.layout.left_panel = TabGroupSpec::new(
        "panel-left",
        Some("spaces"),
        vec![
            text_tab("spaces", "panel-left", "Spaces", "Personal, Team, Archive", false),
            text_tab(
                "bookmarks",
                "panel-left",
                "Bookmarks",
                "Pinned notes and saved searches.",
                false,
            ),
        ],
    );

    product.layout.right_panel = TabGroupSpec::new(
        "panel-right",
        Some("details"),
        vec![
            text_tab(
                "details",
                "panel-right",
                "Details",
                "Metadata, tags, and collaborators.",
                false,
            ),
            text_tab(
                "history",
                "panel-right",
                "History",
                "Recent edits and timeline events.",
                false,
            ),
        ],
    );

    product.layout.bottom_panel = TabGroupSpec::new(
        "panel-bottom",
        Some("activity"),
        vec![
            text_tab(
                "activity",
                "panel-bottom",
                "Activity",
                "Automations, sync jobs, and notifications.",
                false,
            ),
            text_tab(
                "diagnostics",
                "panel-bottom",
                "Diagnostics",
                "Parser output and indexing warnings.",
                false,
            ),
        ],
    );

    product.layout.workbench = WorkbenchNodeSpec::Split {
        axis: SplitAxis::Vertical,
        children: vec![
            WorkbenchNodeSpec::Group(TabGroupSpec::new(
                "workbench-main",
                Some("today"),
                vec![
                    text_tab(
                        "today",
                        "workbench-main",
                        "Today",
                        "Daily dashboard for notes, tasks, and drafts.",
                        false,
                    ),
                    text_tab(
                        "draft",
                        "workbench-main",
                        "Draft",
                        "Write here. This stays intentionally neutral in the example.",
                        true,
                    ),
                ],
            )),
            WorkbenchNodeSpec::Group(TabGroupSpec::new(
                "workbench-secondary",
                Some("board"),
                vec![
                    text_tab(
                        "board",
                        "workbench-secondary",
                        "Board",
                        "Secondary workbench area for planning and structure.",
                        false,
                    ),
                    text_tab(
                        "review",
                        "workbench-secondary",
                        "Review",
                        "Compare revisions, notes, or external context here.",
                        true,
                    ),
                ],
            )),
        ],
    };

    let config = MaruzzellaConfig::new("com.example.notebook")
        .with_persistence_id("notebook-example")
        .with_product(product);

    run(config);
}
