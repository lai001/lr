use rs_artifact::material_paramenters::{BaseDataValueType, StructField};
use rs_core_minimal::name_generator::make_unique_name;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Paramenters {
    fields: Vec<StructField>,
}

impl Paramenters {
    pub fn empty() -> Self {
        Self { fields: vec![] }
    }

    pub fn add(&mut self, name: String, data_type: BaseDataValueType) -> bool {
        if name.is_empty() {
            return false;
        }
        let name = make_unique_name(self.fields.iter().map(|x| x.name.clone()).collect(), name);
        let field = StructField { name, data_type };
        self.fields.push(field);
        true
    }

    pub fn fields(&self) -> &[StructField] {
        &self.fields
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let num = self.fields.len();
        self.fields.retain(|x| x.name != name);
        num != self.fields.len()
    }

    pub fn fields_iter_mut(&mut self) -> std::slice::IterMut<'_, StructField> {
        self.fields.iter_mut()
    }

    pub fn change_type(&mut self, name: &str, new_type: BaseDataValueType) -> bool {
        for field in &mut self.fields {
            if field.name == name {
                field.data_type = new_type;
                return true;
            }
        }
        return false;
    }

    pub fn change_name(&mut self, old_name: &str, new_name: &str) -> bool {
        let unique_name = make_unique_name(
            self.fields.iter().map(|x| x.name.clone()).collect(),
            new_name,
        );
        for field in &mut self.fields {
            if field.name == old_name {
                field.name = unique_name;
                return true;
            }
        }
        return false;
    }

    pub fn is_valid(&self) -> bool {
        !self.fields.is_empty()
    }
}
