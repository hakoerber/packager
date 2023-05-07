use super::Tree;
use axohtml::html;

pub struct Home {
    doc: Tree,
}

impl Home {
    pub fn build() -> Self {
        let doc = html!(
            <div id="home" class=["p-8", "max-w-xl"]>
                <p><a href="/inventory">"Inventory"</a></p>
                <p><a href="/trips">"Trips"</a></p>
            </div>
        );

        Self { doc }
    }
}

impl Into<Tree> for Home {
    fn into(self) -> Tree {
        self.doc
    }
}
