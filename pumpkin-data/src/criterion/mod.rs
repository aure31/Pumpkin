use crate::Advancement;

struct CriterionRegistry<F: Fn(&'static Advancement)> {
    grant_advancement: F,
}
