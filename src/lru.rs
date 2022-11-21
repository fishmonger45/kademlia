const LRU_SIZE: usize = 20;

pub struct Lru<T>(pub Vec<T>);

impl<T> Lru<T>
where
    T: PartialEq + Eq,
{
    fn new() -> Self {
        Lru(Vec::<T>::with_capacity(LRU_SIZE))
    }

    /// Upsert a value in a [`Lru`]. Moving existing values to the tail
    fn upsert(&mut self, x: T) {
        if !self.0.contains(&x) {
            self.0.remove(
                self.0
                    .iter()
                    .position(|y| *y == x)
                    .expect("needle not found"),
            );
        }
        self.0.push(x);
    }

    /// Check if the node is contained within the [`Lru`]
    fn contains(&self, x: &T) -> bool {
        self.0.iter().any(|y| y == x)
    }

    /// Remove a given element from the [`Lru`]
    fn remove(&mut self, x: &T) -> Option<T> {
        self.0.iter().position(|y| y == x).map(|y| self.0.remove(y))
    }


}

mod test {}
