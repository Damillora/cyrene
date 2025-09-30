use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Color, Style,
        object::{Columns, Rows},
        themes::Colorization,
    },
};

use crate::responses::CyreneAppItem;

#[derive(Tabled)]
#[tabled(rename_all = "Upper Title Case")]
pub struct CyreneAppVersionsRow {
    pub name: String,
    pub version: String,
}

impl From<&(String, String)> for CyreneAppVersionsRow {
    fn from(value: &(String, String)) -> Self {
        CyreneAppVersionsRow {
            name: value.0.clone(),
            version: value.1.to_string(),
        }
    }
}

impl From<&CyreneAppItem> for CyreneAppVersionsRow {
    fn from(value: &CyreneAppItem) -> Self {
        CyreneAppVersionsRow {
            name: value.name.clone(),
            version: value.version.clone(),
        }
    }
}

pub fn cyrene_app_versions(versions: &[(String, String)], long_ver: bool) {
    if long_ver {
        let table_items = versions.iter().map(CyreneAppVersionsRow::from);

        let mut table = Table::new(table_items);
        table.with(Style::blank());
        table.with(Colorization::exact([Color::FG_BRIGHT_BLUE], Rows::first()));
        table.modify(Columns::first(), Alignment::left());

        println!("{}", table);
    } else {
        versions.iter().for_each(|f| println!("{}", f.1));
    }
}

pub fn cyrene_app_versions_all(versions: &[CyreneAppVersionsRow], long_ver: bool) {
    if long_ver {
        let table_items = versions.iter();

        let mut table = Table::new(table_items);
        table.with(Style::blank());
        table.with(Colorization::exact([Color::FG_BRIGHT_BLUE], Rows::first()));
        table.modify(Columns::first(), Alignment::left());

        println!("{}", table);
    } else {
        versions
            .iter()
            .for_each(|f| println!("{}: {}", f.name, f.version));
    }
}
