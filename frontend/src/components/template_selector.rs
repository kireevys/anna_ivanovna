use yew::prelude::*;

use ai_core::plan::Plan;

use crate::api::types::{Collection, CollectionContent, Tag};

#[derive(Properties, PartialEq)]
pub struct TemplateSelectorProps {
    pub collections: Vec<Collection>,
    pub on_select: Callback<Plan>,
    pub on_create_empty: Callback<()>,
}

#[function_component(TemplateSelector)]
pub fn template_selector(props: &TemplateSelectorProps) -> Html {
    html! {
        <div class="flex flex-col gap-10">
            <div class="text-center">
                <h1 class="text-3xl font-bold mb-3">{"Создайте свой первый план"}</h1>
                <p class="text-base-content/60 max-w-lg mx-auto mb-4">
                    {"Мы подготовили стратегии на основе проверенных временем принципов. Выберите ту, что ближе вашей ситуации — потом настроите под себя."}
                </p>
                { render_create_empty(&props.on_create_empty) }
            </div>
            { for props.collections.iter().map(|c| render_collection(c, &props.on_select)) }
        </div>
    }
}

fn render_collection(collection: &Collection, on_select: &Callback<Plan>) -> Html {
    html! {
        <div class="flex flex-col gap-6">
            { render_collection_header(collection) }
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                { for collection.templates.iter().map(|t| render_template_card(t, on_select)) }
            </div>
        </div>
    }
}

fn render_collection_header(collection: &Collection) -> Html {
    let links = match &collection.content {
        CollectionContent::Book {
            book_url,
            audio_url,
        } => html! {
            <div class="flex gap-4 text-sm justify-center">
                <a href={book_url.clone()} target="_blank" rel="noopener" class="link link-primary">
                    {"Читать"}
                </a>
                <a href={audio_url.clone()} target="_blank" rel="noopener" class="link link-primary">
                    {"Слушать"}
                </a>
            </div>
        },
    };

    html! {
        <div class="text-center">
            <h2 class="text-2xl font-bold mb-2">{ &collection.name }</h2>
            <p class="text-base-content/70 mb-3 whitespace-pre-line">{ &collection.description }</p>
            { links }
        </div>
    }
}

fn badge_text(tag: &Tag) -> &'static str {
    match tag {
        Tag::Recommended => "Рекомендуем",
        Tag::Stability => "Стабильность",
        Tag::Debt => "Долги",
        Tag::Future => "Будущее",
    }
}

fn badge_class(tag: &Tag) -> &'static str {
    match tag {
        Tag::Recommended => "badge badge-primary",
        Tag::Stability => "badge badge-success",
        Tag::Debt => "badge badge-warning",
        Tag::Future => "badge badge-info",
    }
}

fn render_template_card(
    template: &crate::api::types::PlanTemplate,
    on_select: &Callback<Plan>,
) -> Html {
    let plan = template.plan.clone();
    let on_click = {
        let on_select = on_select.clone();
        Callback::from(move |_: MouseEvent| {
            on_select.emit(plan.clone());
        })
    };

    let is_recommended = template.tag == Tag::Recommended;
    let card_class = if is_recommended {
        "card card-bordered border-primary border-2 cursor-pointer hover:shadow-lg transition-shadow"
    } else {
        "card card-bordered cursor-pointer hover:shadow-lg transition-shadow"
    };

    html! {
        <div class={card_class} onclick={on_click}>
            <div class="card-body">
                <div class="flex justify-between items-start">
                    <div>
                        <h3 class="card-title">{ &template.name }</h3>
                        <p class="text-sm text-base-content/50">{ &template.subtitle }</p>
                    </div>
                    <span class={badge_class(&template.tag)}>{ badge_text(&template.tag) }</span>
                </div>
                <p class="text-sm italic text-base-content/70 mt-2">{ &template.situation }</p>
                <p class="text-sm mt-2">{ &template.description }</p>
                <div class="card-actions justify-end mt-2">
                    <span class="text-lg font-mono font-bold">{ &template.tagline }</span>
                </div>
            </div>
        </div>
    }
}

fn render_create_empty(on_create_empty: &Callback<()>) -> Html {
    let on_click = {
        let cb = on_create_empty.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    html! {
        <button class="btn btn-outline btn-sm" onclick={on_click}>
            {"Настрою сам"}
        </button>
    }
}
