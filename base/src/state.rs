use crate::loader::DataLoader;
use crate::store::DataStore;
pub fn init() -> io::Result<State> {
    let store = DataStore::open()?;
    let loader = DataLoader::new(store);

    Ok(State { loader })
}
