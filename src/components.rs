use super::Plant;
use leptos::*;

#[component]
pub fn Index(cx: Scope, plants: Vec<Plant>) -> impl IntoView {
    view! { cx,
        <head>
            <script src="https://unpkg.com/htmx.org@1.9.2" integrity="sha384-L6OqL9pRWyyFU3+/bjdSri+iIphTN/bvYyM37tICVyOJkWZLpP2vGn6VUEXgzg6h" crossorigin="anonymous"></script>
            <script src="https://unpkg.com/htmx.org/dist/ext/json-enc.js"></script>
            <link href="https://fonts.googleapis.com/css?family=Roboto:100,300,400,500,700,900" rel="stylesheet"/>
            <link rel="stylesheet" href="./css/styles.css"/>
        </head>
        <body>
            <MainView
                plants=plants
            />
        </body>
    }
}

#[component]
pub fn MainView(cx: Scope, plants: Vec<Plant>) -> impl IntoView {
    view! {cx,
        <div class="site-wrapper">
            <div class="button-bar">
                <div class="button-bar-child">
                    <button id="view-button"
                        class="main-buttons"
                        hx-get="/plant-view"
                        hx-trigger="click"
                        hx-target="#main-view"
                        hx-swap="innerHTML"
                    >"View plants"</button>

                    <button id="add-button"
                        class="main-buttons"
                        hx-get="/add-view"
                        hx-trigger="click"
                        hx-target="#main-view"
                        hx-swap="innerHTML"
                    >"Add plant"</button>
                    <button id="view-attn-needed-button"
                        class="main-buttons"
                        hx-get="/get-plants-that-need-attention"
                        hx-trigger="click"
                        hx-target="#main-view"
                        hx-swap="innerHTML"
                    >"See plants that need attention"</button>
                </div>
                <div class="button-bar-child button-bar-child-right">
                    <button id="sort-by-feed"
                        hx-get="/sort-by-feed"
                        hx-trigger="click"
                        hx-target="#plants"
                        hx-swap="outerHTML"
                    >"Sort by least recently fed"</button>
                    <button id="sort-by-pot"
                        hx-get="/sort-by-pot"
                        hx-trigger="click"
                        hx-target="#plants"
                        hx-swap="outerHTML"
                    >"Sort by least recently potted"</button>
                    <button id="sort-by-prune"
                        hx-get="/sort-by-prune"
                        hx-trigger="click"
                        hx-target="#plants"
                        hx-swap="outerHTML"
                    >"Sort by least recently pruned"</button>
                    <div class="search-bar">
                        <input
                            class="search-input"
                            type="search"
                            name="search_string"
                            placeholder="Type to search plants"
                            hx-post="/search-plants"
                            hx-trigger="keyup changed delay:500ms, search"
                            hx-target="#plants"
                            hx-swap="outerHTML"
                            hx-indicator=".htmx-indicator"
                        />
                        <span class="htmx-indicator">"Searching..."</span> //TODO - animated svg

                    </div>
                </div>
            </div>
            <main id="main-view">
                <PlantView
                    plants=plants
                />
            </main>
        </div>

    }
}

#[component]
pub fn PlantAddSuccess(cx: Scope, plant: Plant) -> impl IntoView {
    view! {cx,
        <p>"Plant added successfully"</p>
        <PlantItem
            plant=plant
        />
    }
}

#[component]
pub fn PlantAddFailure(cx: Scope, error: String) -> impl IntoView {
    view! {cx,
        <p>"Plant not added! Please trying adding it again"</p>
        <p>"The following error code was encounted:" {move || error.clone()}</p>
    }
}

/// Form for adding plants, user_id is prefilled on server.
#[component]
pub fn AddPlantView(cx: Scope, user_id: i32, plant_id: Option<i32>, text: String) -> impl IntoView {
    let plant_id = plant_id.unwrap_or(-1);
    view! { cx,
        <div id="add-view">
            <form>
                <input type="hidden" name="plant_id" value=plant_id/>
                <input type="hidden" name="user_id" value=user_id.to_string()/>
                <label for="botanical_name">Botanical name: </label>
                <input type="text" name="botanical_name" id="botanical_name" required />

                <label for="common_name">Common name: </label>
                <input type="text" name="common_name" id="common_name" required />

                <div>
                    <label for="feed_optional">Is fertilizing optional?</label>
                    <input type="checkbox" name="feed_optional" id="feed_optional"/>
                </div>
                <label class="feed-vis" for="last_fed">Last fertilized: </label>
                <input class="feed-vis" type="date" name="last_fed" id="last_fed" required />
                <label class="feed-vis" for="feed_interval">Fertilizing interval in days: </label>
                <input class="feed-vis" type="number" name="feed_interval" id="feed_interval" required />

                <div>
                    <label for="pot_optional">Is potting optional?</label>
                    <input type="checkbox" name="pot_optional" id="pot_optional"/>
                </div>
                <label class="pot-vis" for="last_potted">Last Potted: </label>
                <input class="pot-vis" type="date" name="last_potted" id="last_potted" required />
                <label class="pot-vis" for="potting_interval">Potting interval in days: </label>
                <input class="pot-vis" type="number" name="potting_interval" id="potting_interval" required />

                <div>
                    <label for="prune_optional">Is pruning optional?</label>
                    <input type="checkbox" name="prune_optional" id="prune_optional"/>
                </div>
                <label class="prune-vis" for="last_pruned">Last pruned: </label>
                <input class="prune-vis" type="date" name="last_pruned" id="last_pruned" required />
                <label class="prune-vis" for="pruning_interval">Pruning interval in days: </label>
                <input class="prune-vis" type="number" name="pruning_interval" id="pruning_interval" required />

                <input type="submit"
                    value=text
                    hx-post="/add-plant"
                    hx-trigger="click"
                    hx-target="#add-view"
                    hx-swap="outerHTML"
                    />

            </form>
            <script>
            r#"
                document.getElementById('feed_optional').addEventListener('change', (event) => {
                    toggleFields('feed-vis', event.target.checked);
                });    
                document.getElementById('pot_optional').addEventListener('change', (event) => {
                    toggleFields('pot-vis', event.target.checked);
                });    
                document.getElementById('prune_optional').addEventListener('change', () => {
                    toggleFields('prune-vis', event.target.checked);
                });    

                function toggleFields(className, isChecked) {
                    console.log('pressed ' + className);
                    Array.from(document.getElementsByClassName(className)).forEach((field) => {
                        console.log('is checked: ' + isChecked);
                        if (isChecked) {
                            if (field.tagName == 'input') {
                                if (field.type == 'date') {
                                    field.value = '3000-11-11';
                                } else if (field.type == 'number') {
                                    field.value = 100000;
                                }
                            }
                            field.style.display = 'none';
                        } else {
                            if (field.tagName == 'input') {
                                if (field.type == 'date') {
                                    field.value = '';
                                } else if (field.type == 'number') {
                                    field.value = '';
                                }
                            }
                            field.style.display = 'block';
                        }
                    });
                }

            "#
            </script>
        </div>

    }
}

#[component]
pub fn PlantView(cx: Scope, plants: Vec<Plant>) -> impl IntoView {
    let (plants, _) = create_signal(cx, plants);

    view! { cx,
        <ul id="plants" class="plant-view">
            <For
                //get each item we iterate over
                each=move || plants.get()
                key=|plant| plant.plant_id
                view=move |cx, plant: Plant| {
                    view! { cx,
                        <li><PlantItem plant=plant /></li>
                    }
                }
            />
        </ul>
    }
}

#[component]
pub fn PlantItem(cx: Scope, plant: Plant) -> impl IntoView {
    let (feed_days, pot_days, prune_days) = super::get_days_till_next_feed(&plant);
    let format = time::format_description::parse("[year]-[month]-[day]").unwrap();
    let feed_date = (plant.last_fed.format(&format)).unwrap();
    let pot_date = (plant.last_potted.format(&format)).unwrap();
    let prune_date = (plant.last_pruned.format(&format)).unwrap();
    //let plant_id_json = format!(r#"{{"plant_id": {} }}"#, plant.plant_id);

    view! { cx,
        <div class="plant-container">
            <div>Plant id: {plant.plant_id}</div>
            <div>Botanical name: {plant.botanical_name}</div>
            <div>Common name: {plant.common_name}</div>
            <div>Last fed: {feed_date}</div>
            <div>Time to next feeding cycle: {feed_days}</div>
            <div>Last potted: {pot_date}</div>
            <div>Time to next potting cycle: {pot_days}</div>
            <div>Last pruned: {prune_date}</div>
            <div>Time to next pruning cycle: {prune_days}</div>
            <form>
                <input type="number" name="plant_id" hidden="true" value=plant.plant_id/>
                <input type="submit"
                value="Update plant"
                hx-post="/update-view"
                hx-trigger="click"
                hx-target="#main-view"
                hx-swap="innerHTML"
                />
            </form>
        </div>
    }
}

#[component]
pub fn EmailItem(cx: Scope, plants: Vec<Plant>) -> impl IntoView {
    view! { cx,
        <h1>"These plants require attention!"</h1>
        <h3>"Sorry to bother you but your little green plant friends require some attention!"</h3>
        <PlantView
            plants=plants
        />
    }
}

#[component]
pub fn UpdateView(cx: Scope, plant: Plant, user_id: i32) -> impl IntoView {
    let format = time::format_description::parse("[year]-[month]-[day]").unwrap();
    let feed_date = (plant.last_fed.format(&format)).unwrap();
    let pot_date = (plant.last_potted.format(&format)).unwrap();
    let prune_date = (plant.last_pruned.format(&format)).unwrap();

    view! { cx,
        <h2>"Details to Update"</h2>
        <div class="plant-container">
            <form
                hx-post="/update-plant"
                hx-trigger="submit"
                hx-target="#main-view"
                hx-swap="innerHTML"
            >
                <div>
                <label>
                    <input type="number" name="user_id" hidden="true" value=user_id/>
                    "Plant id: "
                    <input type="number" name="plant_id" readonly value=plant.plant_id />
                </label>
                </div>
                <div>
                <label>
                    "Botanical name: "
                    <input type="text" name="botanical_name" value=plant.botanical_name />
                </label>
                </div>
                <div>
                <label>
                    "Common name: "
                    <input type="text" name="common_name" value=plant.common_name />
                </label>
                </div>
                <div>
                <label>
                    "Last fed: "
                    <input type="date" name="last_fed" value=feed_date/>
                </label>
                </div>
                <div>
                <label>
                    "Feed interval: "
                    <input type="number" name="feed_interval" value=plant.feed_interval />
                </label>
                </div>
                <div>
                <label>
                    "Last potted: "
                    <input type="date" name="last_potted" value=pot_date />
                </label>
                </div>
                <div>
                <label>
                    "Time to next potting cycle: "
                    <input type="number" name="potting_interval" value=plant.potting_interval/>
                </label>
                </div>
                <div>
                <label>
                    "Last pruned: "
                    <input type="date" name="last_pruned"  value=prune_date/>
                </label>
                </div>
                <div>
                <label>
                    "Time to next pruning cycle: "
                    <input type="number" name="pruning_interval" value=plant.pruning_interval />
                </label>
                </div>
                <input type="submit" value="Update plant"/>
            </form>
        </div>
    }
}

#[component]
pub fn LoginView(cx: Scope) -> impl IntoView {
    view! {cx,
        <form
            id="login-form"
            hx-post="/login-email"
            hx-trigger="submit"
            hx-target="#not-logged-in-main"
            hx-swap="outerHTML"
        >
            <label>
                "Username: "
                <input type="text" name="email"/>
            </label>
            <label>
                "Password: "
                <input type="password" name="password"/>
            </label>
            <input type="submit" value="Submit"/>
        </form>
    }
}

#[component]
pub fn NotLoggedInMain(cx: Scope) -> impl IntoView {
    view! { cx,
        <head>
            <script src="https://unpkg.com/htmx.org@1.9.2" integrity="sha384-L6OqL9pRWyyFU3+/bjdSri+iIphTN/bvYyM37tICVyOJkWZLpP2vGn6VUEXgzg6h" crossorigin="anonymous"></script>
            <script src="https://unpkg.com/htmx.org/dist/ext/json-enc.js"></script>
            <link href="https://fonts.googleapis.com/css?family=Roboto:100,300,400,500,700,900" rel="stylesheet"/>
            <link rel="stylesheet" href="./css/styles.css"/>
        </head>
        <body>
            <div id="not-logged-in-main">
                <h1>"Welcome to Plant Track!"</h1>
                <h2>"Please login or sign up to continue"</h2>
                <LoginView/>
                <SignUpForm/>
            </div>
        </body>
    }
}

#[component]
pub fn SignUpForm(cx: Scope) -> impl IntoView {
    view! { cx,
        <form
            id="signup-form"
            action="/signup"
            method="post"
        >
            <input type="hidden" name="user_id" value="-1"/>
            <label>
                "First name: "
                <input type="text" name="first_name" required=true/>
            </label>
            <label>
                "Last name: "
                <input type="text" name="last_name" required=true/>
            </label>
            <label>
                "Email Address: "
                <input type="text" name="email" required=true/>
            </label>
            <label>
                "Password: "
                <input type="password" name="password_hash" required=true/>
            </label>
            <input
            type="submit"
            value="Submit"
            onclick="this.disabled=true;this.value='Sending, please wait...';this.form.submit();"
            />
        </form>
    }
}
