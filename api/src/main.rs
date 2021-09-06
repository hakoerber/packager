#[derive(Debug)]
enum ItemUsage {
    Singleton,
    Once,
    Periodic(i32),
}

#[derive(Debug)]
struct PackageItem {
    name: String,
    usage: ItemUsage
}

impl PackageItem {
    fn new(name: String, usage: ItemUsage) -> PackageItem {
        PackageItem{
            name: name,
            usage: usage,
        }
    }
}

struct PackageList<'a> {
    name: String,
    items: &'a[PackageItem],
}

fn main() {
    let package_list_camping = PackageList {
        name: String::from("camping"),
        items: &[
            PackageItem::new(String::from("Schlafsack"), ItemUsage::Singleton),
            PackageItem::new(String::from("Zelt"), ItemUsage::Singleton),
            PackageItem::new(String::from("Luftmatratze"), ItemUsage::Singleton),
            PackageItem::new(String::from("Campingstuhl"), ItemUsage::Singleton),
            PackageItem::new(String::from("Regenjacke"), ItemUsage::Singleton),
            PackageItem::new(String::from("Tasse"), ItemUsage::Singleton),
            PackageItem::new(String::from("Topf"), ItemUsage::Singleton),
            PackageItem::new(String::from("Messer"), ItemUsage::Singleton),
            PackageItem::new(String::from("Messer"), ItemUsage::Singleton),
            PackageItem::new(String::from("Unterhose"), ItemUsage::Periodic(1)),
        ],
    };

    println!("Contents for package list {:?}:", package_list_camping.name);
    for item in package_list_camping.items {
        println!("{:?}", item);
    }
}
