use super::Tree;
use crate::models::*;

use axohtml::{html, text};

pub struct TripList {
    doc: Tree,
}

impl TripList {
    pub fn build(package_lists: Vec<Trip>) -> Self {
        let doc = html!(
            <table>
                <thead>
                    <tr>
                        <th>"ID"</th>
                        <th>"Name"</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        package_lists.into_iter().map(|list| {
                            html!(
                                <tr>
                                    <td>{text!(list.id.to_string())}</td>
                                    <td>{text!(list.name)}</td>
                                </tr>
                            )
                        })
                    }
                </tbody>
            </table>
        );

        Self { doc }
    }
}

impl Into<Tree> for TripList {
    fn into(self) -> Tree {
        self.doc
    }
}
