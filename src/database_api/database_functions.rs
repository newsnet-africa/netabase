use super::DataType;

fn add<T: DataType>(data: T) {
    let key = data.key();
}
