mod browser;
mod payload;

use browser::Browser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut b = Browser::new()?;
    println!("{:#?}", b.navigate("example.json").await?);
    println!("{:#?}", b.navigate("example2.json").await?);
    println!("{:?}", webbrowser::open("http://github.com"));
    Ok(())
}
