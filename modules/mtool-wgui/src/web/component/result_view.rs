// use std::{error::Error, rc::Rc};

// use yew::prelude::*;

// #[derive(Properties)]
// pub struct Props<COMP, T, E>
// where
//     COMP: BaseComponent<Properties = T>,
//     T: Properties,
// {
//     pub child: ChildrenWithProps<COMP>,
//     pub result: Result<T, E>,
// }

// impl<COMP, T, E> PartialEq for Props<COMP, T, E>
// where
//     COMP: BaseComponent<Properties = T>,
//     T: Properties,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.child == other.child
//     }
// }

// #[function_component]
// pub fn ResultView<COMP, T, E>(props: &Props<COMP, T, E>) -> Html
// where
//     COMP: BaseComponent<Properties = T>,
//     T: Properties + Clone,
//     E: Error,
// {
//     match &props.result {
//         Ok(value) => html! {
//             {
//                 for props.child.iter().map(|mut item| {
//                     let props = Rc::make_mut(&mut item.props);
//                     *props = value.clone();
//                     item
//                 })
//             }
//         },
//         Err(e) => html! {
//             <div>
//               { e.to_string() }
//             </div>
//         },
//     }
// }
