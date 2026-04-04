use yew::prelude::*;

/// Lucide icons (https://lucide.dev) as inline SVG components.
/// Only icons actually used in the app are included.

#[derive(Properties, PartialEq)]
pub struct IconProps {
    #[prop_or("w-4 h-4".to_string())]
    pub class: String,
}

/// Lucide "landmark" — bank/institution building
#[function_component(LandmarkIcon)]
pub fn landmark_icon(props: &IconProps) -> Html {
    html! {
        <svg class={props.class.clone()} viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <line x1="3" y1="22" x2="21" y2="22"/>
            <line x1="6" y1="18" x2="6" y2="11"/>
            <line x1="10" y1="18" x2="10" y2="11"/>
            <line x1="14" y1="18" x2="14" y2="11"/>
            <line x1="18" y1="18" x2="18" y2="11"/>
            <polygon points="12 2 20 7 4 7 12 2"/>
            <line x1="3" y1="11" x2="21" y2="11"/>
        </svg>
    }
}

/// Lucide "mail" — envelope
#[function_component(MailIcon)]
pub fn mail_icon(props: &IconProps) -> Html {
    html! {
        <svg class={props.class.clone()} viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <rect width="20" height="16" x="2" y="4" rx="2"/>
            <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>
        </svg>
    }
}

/// Lucide "x" — close/delete
#[function_component(XIcon)]
pub fn x_icon(props: &IconProps) -> Html {
    html! {
        <svg class={props.class.clone()} viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 6 6 18"/>
            <path d="m6 6 12 12"/>
        </svg>
    }
}
