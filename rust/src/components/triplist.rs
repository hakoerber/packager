use crate::models::*;

use maud::{html, Markup};

pub struct TripList {
    doc: Markup,
}

impl TripList {
    pub fn build(package_lists: Vec<Trip>) -> Self {
        let doc = html!(
            table {
                thead {
                    td {
                        td { "ID" }
                        td { "Name" }
                    }
                }
                tbody {
                    @for list in package_lists {
                        tr {
                            td { (list.id.to_string()) }
                            td { (list.name) }
                        }
                    }
                }
            }
        );

        Self { doc }
    }
}

impl From<TripList> for Markup {
    fn from(val: TripList) -> Self {
        val.doc
    }
}
