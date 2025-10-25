use crate::field_type::FieldType;

/// Defines a field in a line record.
///
/// The Field definition is eventually used in comparison.
///
/// # Examples
/// ```
/// // specify that the second field of the record is a String and that its blanks to be stripped
/// // and case ignored for comparison
/// use text_file_sort::field::Field;
/// use text_file_sort::field_type::FieldType;
/// let field = Field::new(2, FieldType::String)
///     .with_ignore_blanks(true)
///     .with_ignore_case(true);
/// ```
#[derive(Clone, Debug)]
pub struct Field {
    name: String,
    index: usize,
    field_type: FieldType,
    ignore_blanks: bool,
    ignore_case: bool,
    random: bool,
}

impl Field {
    /// Create a new [Field]
    ///
    /// # Arguments
    /// * `index` - the index of the field, starting at 1. Index of 0 treats the complete line as a
    ///   field
    /// * `field_type` - the type of the field. See [FieldType] for supported types
    ///
    /// # Examples
    /// ```
    /// use text_file_sort::field::Field;
    /// use text_file_sort::field_type::FieldType;
    /// let field = Field::new(1, FieldType::Integer);
    /// ```
    pub fn new(
        index: usize,
        field_type: FieldType,
    ) -> Field {
        Field {
            name: String::new(),
            index,
            field_type,
            ignore_blanks: false,
            ignore_case: false,
            random: false,
        }
    }

    /// Get the name for this field.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Get the index for this field.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the [FieldType] for this field.
    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }

    /// Get the ignore blanks setting for this field
    pub fn ignore_blanks(&self) -> bool {
        self.ignore_blanks
    }

    /// Get the ignore case setting for this field.
    pub fn ignore_case(&self) -> bool {
        self.ignore_case
    }

    /// Get the random setting for this field.
    pub fn random(&self) -> bool {
        self.random
    }

    /// Specify a name for this field
    pub fn with_name(mut self, name: String) -> Field {
        self.name = name;
        self
    }

    /// Specify a name for this field as &str
    pub fn with_str_name(mut self, name: &str) -> Field {
        self.name = name.to_string();
        self
    }

    /// Specify the index for this field starting at 1. Specifying index of 0 treats the complete
    /// line as a field.
    pub fn with_index(mut self, index: usize) -> Field {
        self.index = index;
        self
    }

    /// Specify the field type for this field. See [FieldType] for supported types.
    pub fn with_field_type(mut self, field_type: FieldType) -> Field {
        self.field_type = field_type;
        self
    }

    /// Specify whether to ignore blanks for comparison. When true the field will be trimmed before
    /// comparison.
    pub fn with_ignore_blanks(mut self, ignore_blanks: bool) -> Field {
        self.ignore_blanks = ignore_blanks;
        self
    }

    /// Specify whether to ignore case for comparison.
    pub fn with_ignore_case(mut self, ignore_case: bool) -> Field {
        self.ignore_case = ignore_case;
        self
    }

    /// Specify whether to generate a random field value. Specifying true will cause the file to
    /// be randomly shuffled.
    pub fn with_random(mut self, random: bool) -> Field {
        self.random = random;
        self
    }
}
