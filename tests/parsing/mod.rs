use tape_device::assembler::program_model::{DataModel, LabelModel, ProgramModel, StringModel};

mod single;

fn new_program_model(name: &str, ver: &str) -> ProgramModel {
    ProgramModel::new(name.to_owned(), ver.to_owned())
}

#[rustfmt::skip]
fn new_program_model_with_data(name: &str, ver: &str, key: &str) -> ProgramModel {
    let mut model = ProgramModel::new(name.to_owned(), ver.to_owned());

    model.data.insert(String::from(key), DataModel::new(String::from(key), vec![], String::new(), 0));

    model
}

#[rustfmt::skip]
fn new_program_model_with_string(name: &str, ver: &str, key: &str) -> ProgramModel {
    let mut model = ProgramModel::new(name.to_owned(), ver.to_owned());

    model.strings.insert(String::from(key), StringModel::new(String::from(key), String::new(), String::new(), 0));

    model
}

#[rustfmt::skip]
fn new_program_model_with_label(name: &str, ver: &str, key: &str) -> ProgramModel {
    let mut model = ProgramModel::new(name.to_owned(), ver.to_owned());

    model.labels.insert(String::from(key), LabelModel::new(String::from(key), None, vec![]));

    model
}
