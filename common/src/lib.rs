// push everywhere except for continious events
pub type Time = f32;

pub fn time() -> Time {
    1.
}

// #[derive(Debug)]
// pub struct Behavior<T> (fn(Time) -> (T, Box<Behavior<T>>));
// fn at<T>(b: &Behavior<T>, t: Time) -> T {
//     b.0(t).0
// }
// fn occs<T>(e: Event<T>, t: Vec<Time>) -> Event<T> {
//     vec!()
// }
