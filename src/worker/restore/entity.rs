use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TableDescription {
    schema: String,
    name: String,
}

impl TableDescription {
    pub fn new(schema: &str, name: &str) -> Self {
        TableDescription {
            schema: schema.into(),
            name: name.into(),
        }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct IndexDescription {
    schema: String,
    name: String,
}

impl IndexDescription {
    pub fn new(schema: &str, name: &str) -> Self {
        IndexDescription {
            schema: schema.into(),
            name: name.into(),
        }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EntityList {
    full_schemas: HashSet<String>,
    table_schemas: HashSet<String>,
    tables: HashSet<TableDescription>,
}

impl EntityList {
    pub fn parse<S>(value: &[S]) -> EntityList
    where
        S: AsRef<str>,
    {
        let mut full_schemas = HashSet::new();
        let mut table_schemas = HashSet::new();
        let mut tables = HashSet::new();

        for entity in value.iter().map(EntityDescription::parse) {
            match entity {
                EntityDescription::Schema { name } => {
                    full_schemas.insert(name);
                }
                EntityDescription::Table { schema, name } => {
                    let table = TableDescription::new(&schema, &name);

                    table_schemas.insert(schema);
                    tables.insert(table);
                }
                EntityDescription::Empty => {}
            }
        }

        full_schemas.retain(|name| !table_schemas.contains(name));

        EntityList {
            full_schemas,
            table_schemas,
            tables,
        }
    }

    pub fn full_schemas(&self) -> &HashSet<String> {
        &self.full_schemas
    }

    pub fn table_schemas(&self) -> &HashSet<String> {
        &self.table_schemas
    }

    pub fn tables(&self) -> &HashSet<TableDescription> {
        &self.tables
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub enum EntityDescription {
    Schema { name: String },
    Table { schema: String, name: String },
    Empty,
}

impl EntityDescription {
    pub fn parse<S>(value: S) -> EntityDescription
    where
        S: AsRef<str>,
    {
        let value = value.as_ref();

        if value.is_empty() {
            EntityDescription::Empty
        } else {
            match value.find('.') {
                None => EntityDescription::Schema { name: value.into() },
                Some(index) => EntityDescription::Table {
                    schema: value[..index].into(),
                    name: value[index + 1..].into(),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EntityDescription;
    use super::EntityList;
    use super::TableDescription;

    #[test]
    fn should_parse_empty_string_to_empty() {
        let database_object = EntityDescription::parse("");

        assert_eq!(EntityDescription::Empty, database_object);
    }

    #[test]
    fn should_parse_schema_name_to_schema() {
        let database_object = EntityDescription::parse("public");

        assert_eq!(
            EntityDescription::Schema {
                name: "public".into()
            },
            database_object
        );
    }

    #[test]
    fn should_parse_schema_dot_table_to_table() {
        let database_object = EntityDescription::parse("public.table");

        assert_eq!(
            EntityDescription::Table {
                schema: "public".into(),
                name: "table".into(),
            },
            database_object
        );
    }

    #[test]
    fn should_remove_duplicate_schema_names() {
        let entities = EntityList::parse(&["", "public", "public"]);

        assert_eq!(
            entities.full_schemas,
            ["public"].iter().map(|&s| s.into()).collect()
        );
    }

    #[test]
    fn should_remove_duplicate_table_schemas() {
        let entities = EntityList::parse(&["", "public.table", "public.toast"]);

        assert_eq!(
            entities.table_schemas,
            ["public"].iter().map(|&s| s.into()).collect()
        );
    }

    #[test]
    fn should_remove_duplicated_tables() {
        let entities = EntityList::parse(&["", "public.table", "public.table"]);

        assert_eq!(
            entities.tables,
            [TableDescription::new("public", "table"),]
                .iter()
                .cloned()
                .collect()
        );
    }
    #[test]
    fn should_parse_string_vector_to_entities() {
        let entities = EntityList::parse(&[
            "",
            "public",
            "test",
            "public.table",
            "public.toast",
            "data.table",
        ]);

        assert_eq!(
            entities.full_schemas,
            ["test"].iter().map(|&s| s.into()).collect()
        );
        assert_eq!(
            entities.table_schemas,
            ["public", "data"].iter().map(|&s| s.into()).collect()
        );
        assert_eq!(
            entities.tables,
            [
                TableDescription::new("public", "table"),
                TableDescription::new("public", "toast"),
                TableDescription::new("data", "table"),
            ]
            .iter()
            .cloned()
            .collect()
        );
    }
}
