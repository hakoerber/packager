use thirtyfour::prelude::*;

use rand::{distributions::Alphanumeric, seq::IteratorRandom, Rng};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use thirtyfour::common::capabilities::firefox::FirefoxPreferences;
use thirtyfour::{FirefoxCapabilities, WebDriver};

use std::time;

use std::future::Future;
use std::process::{Command, Stdio};

const PORT: u16 = 3001;
const BASEURL: &'static str = "http://localhost";

fn url(path: &str) -> String {
    format!("{BASEURL}:{PORT}{path}")
}

#[derive(Debug)]
#[allow(dead_code)]
enum TestError {
    DriverError { message: String },
    CheckError { message: String },
    AppError { message: String },
}

impl From<WebDriverError> for TestError {
    fn from(error: WebDriverError) -> Self {
        Self::DriverError {
            message: error.to_string(),
        }
    }
}

impl From<std::io::Error> for TestError {
    fn from(error: std::io::Error) -> Self {
        Self::AppError {
            message: error.to_string(),
        }
    }
}

fn random_name() -> String {
    let mut rng = rand::thread_rng();
    let length = { 1..=20 }.choose(&mut rng).unwrap();

    let s: String = rng
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    s
}

async fn run_test<T, R>(inner: T) -> Result<(), TestError>
where
    T: FnOnce(WebDriver) -> R,
    R: Future<Output = Result<(), TestError>>,
{
    let event_in_parent = Arc::new((Mutex::new(false), Condvar::new()));
    let event_in_subprocess = Arc::clone(&event_in_parent);

    let prepared_event_in_parent = Arc::new((Mutex::new(false), Condvar::new()));
    let prepared_event_in_subprocess = Arc::clone(&prepared_event_in_parent);

    let app = thread::spawn(move || -> Result<(), TestError> {
        {
            let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../prepare-test-instance.sh");

            println!("[sub] starting prepare script {script}");
            let handle_prepare = Command::new(script)
                .stdin(Stdio::null())
                // .stdout(Stdio::null())
                // .stderr(Stdio::null())
                .output()?;

            assert!(handle_prepare.status.success());
            println!("[sub] preparation ok");
        }

        let mut handle_gecko = {
            let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../run-test-instance.sh");

            println!("[sub] starting script {script}");
            let handle = Command::new("geckodriver")
                .stdin(Stdio::null())
                // .stdout(Stdio::null())
                // .stderr(Stdio::null())
                .spawn()?;
            println!("[sub] geckodriver started");
            handle
        };

        {
            println!("[sub] sending prepared event");
            let (lock, cvar) = &*prepared_event_in_subprocess;
            let mut prepared = lock.lock().unwrap();
            *prepared = true;
            cvar.notify_all();
            println!("[sub] sent prepared event");
        }

        let mut handle_app = {
            let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../run-test-instance.sh");

            println!("[sub] starting script {script}");
            let handle = Command::new(script)
                .arg(PORT.to_string())
                .stdin(Stdio::null())
                // .stdout(Stdio::null())
                // .stderr(Stdio::null())
                .spawn()?;
            println!("[sub] app started");
            handle
        };

        {
            let (lock, cvar) = &*event_in_subprocess;
            let mut done = lock.lock().unwrap();
            while !*done {
                println!("[sub] waiting for done event");
                done = cvar.wait(done).unwrap();
            }
            println!("[sub] done received");
        }

        // at worst, the child already exited, so we don't care about the
        // return code
        println!("[sub] killing app subprocess");
        let _ = handle_app.kill()?;
        handle_app.wait()?;
        println!("[sub] killed app subprocess");

        println!("[sub] killing gecko subprocess");
        let _ = handle_gecko.kill()?;
        handle_gecko.wait()?;
        println!("[sub] killed gecko subprocess");

        println!("[sub] done");

        Ok(())
    });

    thread::spawn(move || {
        let (lock, cvar) = &*prepared_event_in_parent;
        let mut prepared = lock.lock().unwrap();
        while !*prepared {
            println!("waiting for prepared event");
            prepared = cvar.wait(prepared).unwrap();
        }
        println!("prepared received");
    })
    .join()
    .unwrap();

    println!("preparations done, starting tests in 1s");

    thread::sleep(time::Duration::from_secs(1));

    let prefs = FirefoxPreferences::new();
    let mut caps = FirefoxCapabilities::new();
    caps.set_preferences(prefs)?;
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    // it's shitty, but passing references through async closures is even shittier
    // cloning works for closing, so it's good enough
    let driver_handle = driver.clone();

    // call the actual function
    println!("calling test function");
    let result = inner(driver).await;
    // let result: Result<(), TestError> = Ok(());
    println!("test function done");

    println!("deinitializing selenium driver");
    driver_handle.quit().await?;

    // this has to be done in a separate thread, otherwise the condvar handling
    // does not work. it's not sure why.
    thread::spawn(move || {
        // if the child panicked and cannot receive the event, we don't care and just
        // continue to the exit
        println!("sending done event");
        let (lock, cvar) = &*event_in_parent;
        let mut done = lock.lock().unwrap();
        *done = true;
        cvar.notify_all();
        println!("sent done event");
    })
    .join()
    .unwrap();

    println!("waiting for subprocess");
    app.join().map_err(|_| TestError::AppError {
        message: "app panicked".to_string(),
    })??;
    println!("all done");
    Ok(result?)
}

macro_rules! check_eq {
    ($left:expr, $right:expr) => {
        if ($left != $right) {
            return Err(TestError::CheckError {
                message: format!("line {}: {:?} != {:?}", line!(), $right, $left),
            });
        }
    };
}

async fn check_table(
    table: &WebElement,
    head: &Vec<impl AsRef<str>>,
    body: &Vec<Vec<impl AsRef<str>>>,
) -> Result<(), TestError> {
    let table_head = table
        .find(By::Tag("thead"))
        .await?
        .find_all(By::Tag("th"))
        .await?;

    check_eq!(table_head.len(), head.len());

    for (i, h) in table_head.iter().enumerate() {
        check_eq!(h.text().await?, head[i].as_ref());
    }

    let table_rows = table
        .find(By::Tag("tbody"))
        .await?
        .find_all(By::Tag("tr"))
        .await?;

    check_eq!(table_rows.len(), body.len());

    for (row_i, row) in table_rows.iter().enumerate() {
        let columns = row.find_all(By::Tag("td")).await?;

        check_eq!(columns.len(), body[row_i].len());

        for (column_i, column) in columns.iter().enumerate() {
            check_eq!(column.text().await?, body[row_i][column_i].as_ref());
        }
    }
    Ok(())
}

#[tokio::test]
async fn test() -> Result<(), TestError> {
    let mut handle = {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../run-test-instance.sh");

        println!("[sub] starting script {script}");
        let handle = Command::new(script)
            .arg(PORT.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        println!("[sub] started");
        handle
    };

    // at worst, the child already exited, so we don't care about the
    // return code
    println!("[sub] killing subprocess");
    let _ = handle.kill().expect("failed to kill child");
    handle.wait().unwrap();
    println!("[sub] killed subprocess");

    println!("[sub] done");

    // return Ok(());

    run_test(|driver: WebDriver| async move {
        for js_enabled in [true] {
            driver.goto(url("/")).await?;

            check_eq!(driver.title().await?, "Packager");

            let header = driver.find(By::Id("header")).await?;

            let inventory_link = header.find(By::Id("header-link-inventory")).await?;
            check_eq!(inventory_link.text().await?, "Inventory");
            inventory_link.click().await?;

            check_eq!(driver.current_url().await?.as_str(), url("/inventory/"));

            let category_list = driver.find(By::Id("category-list")).await?;

            check_table(
                &category_list,
                &vec!["Name", "Weight"],
                &vec![vec!["Sum", "0"]],
            )
            .await?;

            let new_category_form = driver.find(By::Id("new-category")).await?;

            let new_category_form_submit = new_category_form
                .find(By::Css("input[type='submit']"))
                .await?;

            check_eq!(new_category_form_submit.is_clickable().await?, !js_enabled);

            // insert a few categories

            let mut rows = vec![vec!["Sum".to_string(), "0".to_string()]];

            let iterations = 3;

            for i in 0..iterations {
                let new_category_form = driver.find(By::Id("new-category")).await?;

                let category_name = random_name();

                let new_category_name_input = new_category_form
                    .find(By::Css("input[name='new-category-name']"))
                    .await?;

                check_eq!(new_category_name_input.value().await?, Some(String::new()));

                new_category_name_input.send_keys(&category_name).await?;

                let new_category_form_submit = new_category_form
                    .find(By::Css("input[type='submit']"))
                    .await?;

                check_eq!(new_category_form_submit.is_clickable().await?, true);
                new_category_form_submit.click().await?;

                let category_list = driver.find(By::Id("category-list")).await?;

                rows.insert(i, vec![category_name, "0".to_string()]);

                check_table(&category_list, &vec!["Name", "Weight"], &rows).await?;
            }

            // select one of the new categories and check that it's empty
            let category_list = driver.find(By::Id("category-list")).await?;

            let table_rows = category_list
                .find(By::Tag("tbody"))
                .await?
                .find_all(By::Tag("tr"))
                .await?;

            let id = { 0..iterations }.choose(&mut rand::thread_rng()).unwrap();

            let category_link = &table_rows[id].find_all(By::Tag("td")).await?[0];

            check_eq!(category_link.is_clickable().await?, true);
            category_link.click().await?;

            check_eq!(
                driver
                    .find(By::Id("items"))
                    .await?
                    .text()
                    .await?
                    .to_lowercase()
                    .contains("empty"),
                true
            )
        }

        Ok(())
    })
    .await
}
