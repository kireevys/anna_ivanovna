use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SectionCardProps {
    pub title: AttrValue,
    #[prop_or_default]
    pub header_right: Option<Html>,
    #[prop_or_default]
    pub children: Children,
}

pub struct SectionCard;

impl Component for SectionCard {
    type Message = ();
    type Properties = SectionCardProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let header_right =
            ctx.props().header_right.as_ref().map(
                |content| html! { <div class="space-x-2">{ content.clone() }</div> },
            );

        html! {
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body space-y-4">
                    <div class="flex justify-between items-center">
                        <h2 class="card-title text-2xl">
                            { &ctx.props().title }
                        </h2>
                        {
                            if let Some(right) = header_right {
                                right
                            } else {
                                html! {}
                            }
                        }
                    </div>
                    { for ctx.props().children.iter() }
                </div>
            </div>
        }
    }
}
