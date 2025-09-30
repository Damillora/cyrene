use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Color, Style,
        object::{Columns, Rows},
        themes::Colorization,
    },
};

use crate::responses::CyreneAppVersions;

#[derive(Tabled)]
#[tabled(rename_all = "Upper Title Case")]
pub struct CyreneAppVersionsRow {
    pub name: String,
    pub version: String,
}

impl From<&CyreneAppVersions> for CyreneAppVersionsRow {
    fn from(value: &CyreneAppVersions) -> Self {
        CyreneAppVersionsRow {
            name: value.name.clone(),
            version: value.version.to_string(),
        }
    }
}

pub fn cyrene_app_versions(versions: &[CyreneAppVersions], long_ver: bool) {
    if long_ver {
        let table_items = versions.iter().map(CyreneAppVersionsRow::from);

        let mut table = Table::new(table_items);
        table.with(Style::blank());
        table.with(Colorization::exact([Color::FG_BRIGHT_BLUE], Rows::first()));
        table.modify(Columns::first(), Alignment::left());

        println!("{}", table);
    } else {
        versions.iter().for_each(|f| println!("{}", f.version));
    }
}
