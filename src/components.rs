use super::Plant;
use leptos::*;

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

                    <button id="update-button"
                        class="main-buttons"
                    >"Update plants"</button> //todo - add view plant func.

                    <button id="add-button"
                        class="main-buttons"
                        hx-get="/add-view"
                        hx-trigger="click"
                        hx-target="#main-view"
                        hx-swap="innerHTML"
                    >"Add plant"</button>
                </div>
                <div class="button-bar-child button-bar-child-right">
                    <button id="sort-by-feed">"Sort by fertilizer requirements"</button>
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
pub fn PlantAddSuccess(cx: Scope) -> impl IntoView {
    view! {cx,
        <p>"Plant added successfully"</p>
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
///
#[component]
pub fn AddPlantView(cx: Scope, user_id: i32) -> impl IntoView {
    view! { cx,
        <div id="add-view">
            <form>
                <input type="hidden" name="plant_id" value=-1/>
                <input type="hidden" name="user_id" value=user_id.to_string()/>
                <label for="botanical_name">Botanical name: </label>
                <input type="text" name="botanical_name" id="botanical_name" required />
                <label for="common_name">Common name: </label>
                <input type="text" name="common_name" id="common_name" required />
                <label for="last_fed">Last fertilized: </label>
                <input type="date" name="last_fed" id="last_fed" required />
                <label for="feed_interval">Fertilizing interval in days: </label>
                <input type="number" name="feed_interval" id="feed_interval" required />
                <label for="last_potted">Last Potted: </label>
                <input type="date" name="last_potted" id="last_potted" required />
                <label for="potting_interval">Potting interval in days: </label>
                <input type="number" name="potting_interval" id="potting_interval" required />
                <label for="last_pruned">Last pruned: </label>
                <input type="date" name="last_pruned" id="last_pruned" required />
                <label for="pruning_interval">Pruning interval in days: </label>
                <input type="number" name="pruning_interval" id="pruning_interval" required />
                <input type="submit"
                    hx-post="/add-plant"
                    hx-trigger="click"
                    hx-target="#add-view"
                    hx-swap="outerHTML"
                    >Add new plant</input>

            </form>
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
                        <PlantItem plant=plant />
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

    view! { cx,
        <li class="plant-container">
            <div>Botanical name: {plant.botanical_name}</div>
            <div>Common name: {plant.common_name}</div>
            <div>Last fed: {feed_date}</div>
            <div>Time to next feeding cycle: {feed_days}</div>
            <div>Last potted: {pot_date}</div>
            <div>Time to next potting cycle: {pot_days}</div>
            <div>Last fed: {prune_date}</div>
            <div>Time to next pruning cycle: {prune_days}</div>
        </li>
    }
}
