// mod inner {
// }

// trait Observer<T> {
//     fn on_next(&self, v: T);
//     fn on_complete(&self);
//     fn on_error(&self);
// }

// struct Subject<T> {
//     observers: Vec<Box<dyn Observer<T>>>,
// }

// impl<T> Subject<T> {
//     fn subscribe<Ob: Observer<T>>(&self, ob: Ob) {
//         self.observers.append(Box::new(ob));
//     }

//     fn next(&self, v: T) {
//         self.observers.iter().for_each(|ob| {
//             ob.on_next(v);
//         })
//     }
//     fn error(&self) {
//         self.observers.iter().for_each(|ob| {
//             ob.on_error();
//         })
//     }
//     fn complete(&self) {
//         self.observers.iter().for_each(|ob| {
//             ob.on_complete();
//         })
//     }
// }

// struct Subscription {}

// struct Observable<T> {}
