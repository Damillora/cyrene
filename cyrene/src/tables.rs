use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Color, Panel, Style, Width,
        object::{Columns, Rows},
        themes::{BorderCorrection, Colorization},
    },
};

use crate::{AppVersion, AppVersionAction, AppVersionUpgradeAction};

#[derive(Tabled)]
#[tabled(rename_all = "Upper Title Case")]
pub struct CyreneAppVersionsRow {
    pub name: String,
    pub version: String,
}
#[derive(Tabled)]
#[tabled(rename_all = "Upper Title Case")]
pub struct CyreneAppVersionsUpgradeRow {
    pub name: String,
    pub old_version: String,
    pub new_version: String,
}
#[derive(Tabled)]
#[tabled(rename_all = "Upper Title Case")]
pub struct CyreneAppVersionsAllRow {
    pub name: String,
    pub version: String,
    pub linked: bool,
}

impl From<&(String, String)> for CyreneAppVersionsRow {
    fn from(value: &(String, String)) -> Self {
        CyreneAppVersionsRow {
            name: value.0.clone(),
            version: value.1.to_string(),
        }
    }
}

impl From<&AppVersionAction> for CyreneAppVersionsRow {
    fn from(value: &AppVersionAction) -> Self {
        CyreneAppVersionsRow {
            name: value.name.clone(),
            version: value.version.clone(),
        }
    }
}
impl From<&AppVersion> for CyreneAppVersionsRow {
    fn from(value: &AppVersion) -> Self {
        CyreneAppVersionsRow {
            name: value.name.clone(),
            version: match &value.version {
                Some(ver) => ver.clone(),
                None => "ALL".to_string(),
            },
        }
    }
}
impl From<&AppVersionUpgradeAction> for CyreneAppVersionsRow {
    fn from(value: &AppVersionUpgradeAction) -> Self {
        CyreneAppVersionsRow {
            name: value.name.clone(),
            version: value.new_version.clone(),
        }
    }
}

impl From<&AppVersionUpgradeAction> for CyreneAppVersionsUpgradeRow {
    fn from(value: &AppVersionUpgradeAction) -> Self {
        CyreneAppVersionsUpgradeRow {
            name: value.name.clone(),
            old_version: value.old_version.clone(),
            new_version: value.new_version.clone(),
        }
    }
}

pub fn cyrene_app_versions(versions: &[(String, String)], long_ver: bool) {
    if long_ver {
        let table_items = versions.iter().map(CyreneAppVersionsRow::from);

        let theme = Style::modern();
        let mut table = Table::new(table_items);
        table.with(theme);
        table.with(Colorization::exact(
            [Color::rgb_fg(255, 175, 255)],
            Rows::first(),
        ));
        table.modify(Columns::first(), Alignment::left());

        println!("{}", table);
    } else {
        versions.iter().for_each(|f| println!("{}", f.1));
    }
}

pub fn cyrene_app_versions_all(versions: &[CyreneAppVersionsAllRow], long_ver: bool) {
    if long_ver {
        let table_items = versions.iter();

        let theme = Style::modern();
        let mut table = Table::new(table_items);
        table.with(theme);
        table.with(Colorization::exact(
            [Color::rgb_fg(255, 175, 255)],
            Rows::first(),
        ));
        table.modify(Columns::first(), Alignment::left());

        println!("{}", table);
    } else {
        versions.iter().for_each(|f| {
            println!(
                "{}: {} {}",
                f.name,
                f.version,
                if f.linked { "(*)" } else { "" },
            )
        });
    }
}

pub fn cyrene_app_install(versions: &[AppVersionAction]) {
    let table_items = versions.iter().map(CyreneAppVersionsRow::from);

    let theme = Style::modern();
    let mut table = Table::new(table_items);
    table.with(theme);
    table.with(Panel::header("Apps to be installed"));
    table.with(BorderCorrection::span());
    table.with(Colorization::exact(
        [Color::FG_BRIGHT_GREEN],
        Columns::last(),
    ));
    table.with(Colorization::exact(
        [Color::rgb_fg(255, 175, 255)],
        Rows::one(1),
    ));
    table.modify(Columns::first(), Alignment::left());
    table.modify(Columns::first(), Width::increase(25));

    println!("{}", table);
}

pub fn cyrene_app_upgrade(versions: &[AppVersionUpgradeAction]) {
    let table_items = versions.iter().map(CyreneAppVersionsUpgradeRow::from);

    let theme = Style::modern();
    let mut table = Table::new(table_items);
    table.with(theme);
    table.with(Panel::header("Apps to be upgraded"));
    table.with(BorderCorrection::span());
    table.with(Colorization::exact(
        [Color::FG_BRIGHT_RED],
        Columns::last() - 1,
    ));
    table.with(Colorization::exact(
        [Color::FG_BRIGHT_GREEN],
        Columns::last(),
    ));
    table.with(Colorization::exact(
        [Color::rgb_fg(255, 175, 255)],
        Rows::one(1),
    ));
    table.modify(Columns::first(), Alignment::left());
    table.modify(Columns::first(), Width::increase(25));

    println!("{}", table);
}

pub fn cyrene_app_remove(versions: &[AppVersion]) {
    let table_items = versions.iter().map(CyreneAppVersionsRow::from);

    let theme = Style::modern();
    let mut table = Table::new(table_items);
    table.with(theme);
    table.with(Panel::header("Apps to be uninstalled"));
    table.with(BorderCorrection::span());
    table.with(Colorization::exact([Color::FG_BRIGHT_RED], Columns::last()));
    table.with(Colorization::exact(
        [Color::rgb_fg(255, 175, 255)],
        Rows::one(1),
    ));
    table.modify(Columns::first(), Alignment::left());
    table.modify(Columns::first(), Width::increase(25));
    println!("{}", table);
}

pub fn cyrene_app_install_unneeded(versions: &[AppVersionAction]) {
    let table_items = versions.iter().map(CyreneAppVersionsRow::from);

    let theme = Style::modern();
    let mut table = Table::new(table_items);
    table.with(theme);
    table.with(Panel::header("Apps already installed"));
    table.with(BorderCorrection::span());
    table.with(Colorization::exact(
        [Color::FG_BRIGHT_YELLOW],
        Columns::last(),
    ));
    table.with(Colorization::exact(
        [Color::rgb_fg(255, 175, 255)],
        Rows::one(1),
    ));
    table.modify(Columns::first(), Alignment::left());
    table.modify(Columns::first(), Width::increase(25));

    println!("{}", table);
}

pub fn cyrene_app_upgrade_unneeded(versions: &[AppVersionUpgradeAction]) {
    let table_items = versions.iter().map(CyreneAppVersionsRow::from);

    let theme = Style::modern();
    let mut table = Table::new(table_items);
    table.with(theme);
    table.with(Panel::header("Apps already on latest version"));
    table.with(BorderCorrection::span());
    table.with(Colorization::exact(
        [Color::FG_BRIGHT_YELLOW],
        Columns::last(),
    ));
    table.with(Colorization::exact(
        [Color::rgb_fg(255, 175, 255)],
        Rows::one(1),
    ));
    table.modify(Columns::first(), Alignment::left());
    table.modify(Columns::first(), Width::increase(25));

    println!("{}", table);
}
