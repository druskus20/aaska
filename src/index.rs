pub fn index_html(meta: crate::SiteMetadata) -> String {
    maud::html! {
        html {
            head {
                title { "Aaska" }
                link rel="stylesheet" href="/static/style.css" {}
            }
            body {
                h1 { "Welcome to Aaska!" }
                p { "This is the index page." }
            }
        }
    }
    .0
}
