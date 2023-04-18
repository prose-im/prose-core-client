use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Kind {
    /// The form-processing entity is asking the form-submitting entity to complete a form.
    Form,
    /// The form-submitting entity is submitting data to the form-processing entity. The submission
    /// MAY include fields that were not provided in the empty form, but the form-processing entity
    /// MUST ignore any fields that it does not understand. Furthermore, the submission MAY omit
    /// fields not marked with <required/> by the form-processing entity.
    Submit,
    /// The form-submitting entity has cancelled submission of data to the form-processing entity.
    Cancel,
    /// The form-processing entity is returning data (e.g., search results) to the form-submitting
    /// entity, or the data is a generic data set.
    Result,
}
