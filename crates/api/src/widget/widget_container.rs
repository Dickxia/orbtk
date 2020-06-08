use std::any::TypeId;

use crate::{
    css_engine::*,
    prelude::*,
    utils::{Brush, String16, Thickness},
};

use dces::prelude::{Component, Entity, EntityComponentManager};

/// The `WidgetContainer` wraps the entity of a widget and provides access to its properties, its children properties and its parent properties.
pub struct WidgetContainer<'a> {
    ecm: &'a mut EntityComponentManager<Tree, StringComponentStore>,
    current_node: Entity,
    theme: &'a ThemeValue,
    _theme: &'a crate::theme::Theme
}

impl<'a> WidgetContainer<'a> {
    /// Creates a new widget container for the given `entity`.
    pub fn new(
        root: Entity,
        ecm: &'a mut EntityComponentManager<Tree, StringComponentStore>,
        theme: &'a ThemeValue,
        _theme: &'a crate::theme::Theme
    ) -> Self {
        WidgetContainer {
            ecm,
            current_node: root,
            theme,
            _theme
        }
    }

    /// Gets the entity of the widget.
    pub fn entity(&self) -> Entity {
        self.current_node
    }

    /// Gets the property.
    ///
    /// # Panics
    ///
    /// Panics if the widget does not contains the property.
    pub fn get<P>(&self, key: &str) -> &P
    where
        P: Clone + Component,
    {
        if let Ok(property) = self.ecm.component_store().get::<P>(key, self.current_node) {
            return property;
        }

        panic!(
            "Entity {} does not contain property type {:?} with key: {}",
            self.current_node.0,
            TypeId::of::<P>(),
            key
        );
    }

    /// Gets a mutable reference of the property of type `P`.
    ///
    /// # Panics
    ///
    /// Panics if the widget does not contains the property.
    pub fn get_mut<P>(&mut self, key: &str) -> &mut P
    where
        P: Clone + Component,
    {
        if let Ok(property) = self
            .ecm
            .component_store_mut()
            .get_mut::<P>(key, self.current_node)
        {
            return property;
        }

        panic!(
            "Entity {} does not contain property type {:?}, with key: {}",
            self.current_node.0,
            TypeId::of::<P>(),
            key
        );
    }

    /// Clones the property. If the property does not exists for the widget the
    /// default of the property value will be returned,
    pub fn clone_or_default<P>(&self, key: &str) -> P
    where
        P: Clone + Component + Default,
    {
        if let Ok(property) = self.ecm.component_store().get::<P>(key, self.current_node) {
            return property.clone();
        }

        P::default()
    }

    /// Clones the property.
    ///
    /// # Panics
    ///
    /// Panics if the widget does not contains the property.
    pub fn clone<P>(&self, key: &str) -> P
    where
        P: Clone + Component,
    {
        if let Ok(property) = self.ecm.component_store().get::<P>(key, self.current_node) {
            return property.clone();
        }

        panic!(
            "Entity {} does not contain property type {:?}, with key: {}",
            self.current_node.0,
            TypeId::of::<P>(),
            key
        );
    }

    /// Clones the property of type `P` from the given widget entity. If the entity does
    /// not exists or it doesn't have a component of type `P` `None` will be returned.
    pub fn try_clone<P>(&self, key: &str) -> Option<P>
    where
        P: Clone + Component,
    {
        if let Ok(property) = self.ecm.component_store().get::<P>(key, self.current_node) {
            return Some(property.clone());
        }

        None
    }

    /// Sets the property of type `P`.
    ///
    /// # Panics
    ///
    /// Panics if the widget does not contains the property.
    pub fn set<P>(&mut self, key: &str, value: P)
    where
        P: Component + Clone,
    {
        if let Ok(property) = self
            .ecm
            .component_store_mut()
            .get_mut::<P>(key, self.current_node)
        {
            *property = value;
            return;
        }

        panic!(
            "Entity {} does not contain property type {:?}",
            self.current_node.0,
            TypeId::of::<P>()
        );
    }

    /// Returns `true` if the widget has a property of type `P` otherwise `false`.
    pub fn has<P>(&self, key: &str) -> bool
    where
        P: Clone + Component,
    {
        self.ecm
            .component_store()
            .get::<P>(key, self.current_node)
            .is_ok()
    }

    /// Returns a reference of a property of type `P` from the given widget entity. If the entity does
    /// not exists or it doesn't have a component of type `P` `None` will be returned.
    pub fn try_get<P: Component>(&self, key: &str) -> Option<&P> {
        self.ecm
            .component_store()
            .get::<P>(key, self.current_node)
            .ok()
    }

    /// Returns a mutable reference of a property of type `P` from the given widget entity. If the entity does
    /// not exists or it doesn't have a component of type `P` `None` will be returned.
    pub fn try_get_mut<P: Component>(&mut self, key: &str) -> Option<&mut P> {
        self.ecm
            .component_store_mut()
            .get_mut::<P>(key, self.current_node)
            .ok()
    }

    /// Checks if the given value is equal to the given property.
    pub fn eq<P: Component + PartialEq>(&self, key: &str, other: &P) -> bool {
        if let Some(value) = self.try_get::<P>(key) {
            return value.eq(other);
        }

        false
    }

    fn update_internal_theme_by_state(&mut self, force: bool, entity: &Entity) {
        for child in &(self.ecm.entity_store().children.clone())[&entity] {
            self.update_internal_theme_by_state(force, child);
        }

        self.current_node = *entity;

        if self.try_get::<crate::theme::Selector>("selector").is_some() {
            if let Some(focus) = self.try_clone::<bool>("focused") {
                update_state("focused", focus, self);
            }

            if let Some(selected) = self.try_clone::<bool>("selected") {
                update_state("selected", selected, self);
            }

            if let Some(pressed) = self.try_clone::<bool>("pressed") {
                update_state("pressed", pressed, self);
            }

            if let Some(enabled) = self.try_clone::<bool>("enabled") {
                update_state("disabled", !enabled, self);
            }

            if let Some(text) = self.try_clone::<String16>("text") {
                update_state("empty", text.is_empty(), self);
            }

            if let Some(expanded) = self.try_clone::<bool>("expanded") {
                update_state("expanded", !expanded, self);
            }

            if self.get::<crate::theme::Selector>("selector").dirty || force {
                self.update_properties_by_theme();
            }
        }
    }

    /// Updates the theme by the inner state e.g. `selected` or `pressed`.
    pub fn update_theme_by_state(&mut self, force: bool) {
        self.update_internal_theme_by_state(force, &(self.current_node.clone()));
    }

    /// Update all properties for the theme.
    pub fn update_properties_by_theme(&mut self) {
        if !self.has::<Selector>("selector") {
            return;
        }

        let selector = self.clone::<Selector>("selector");

        if !selector.dirty() {
            return;
        }

        if self.has::<Brush>("foreground") {
            if let Some(color) = self.theme.brush("color", &selector) {
                self.set::<Brush>("foreground", color);
            }
        }

        if self.has::<Brush>("background") {
            if let Some(background) = self.theme.brush("background", &selector) {
                self.set::<Brush>("background", background);
            }
        }

        if self.has::<Brush>("border_brush") {
            if let Some(border_brush) = self.theme.brush("border-color", &selector) {
                self.set::<Brush>("border_brush", border_brush);
            }
        }

        if self.has::<f64>("border_radius") {
            if let Some(radius) = self.theme.uint("border-radius", &selector) {
                self.set::<f64>("border_radius", f64::from(radius));
            }
        }

        if self.has::<f32>("opacity") {
            if let Some(opacity) = self.theme.float("opacity", &selector) {
                self.set::<f32>("opacity", opacity);
            }
        }

        if self.has::<Thickness>("border_width") {
            if let Some(border_width) = self.theme.uint("border-width", &selector) {
                self.set::<Thickness>("border_width", Thickness::from(border_width as f64));
            }
        }

        self.update_font_properties_by_theme(&selector);

        if let Some(mut padding) = self.try_clone::<Thickness>("padding") {
            if let Some(pad) = self.theme.uint("padding", &selector) {
                padding.set_thickness(pad as f64);
            }

            if let Some(left) = self.theme.uint("padding-left", &selector) {
                padding.set_left(left as f64);
            }

            if let Some(top) = self.theme.uint("padding-top", &selector) {
                padding.set_top(top as f64);
            }

            if let Some(right) = self.theme.uint("padding-right", &selector) {
                padding.set_right(right as f64);
            }

            if let Some(bottom) = self.theme.uint("padding-bottom", &selector) {
                padding.set_bottom(bottom as f64);
            }
            self.set::<Thickness>("padding", padding);
        }

        if let Some(mut constraint) = self.try_clone::<Constraint>("constraint") {
            if let Some(width) = self.theme.uint("width", &selector) {
                constraint.set_width(width as f64);
            }

            if let Some(height) = self.theme.uint("height", &selector) {
                constraint.set_height(height as f64);
            }

            if let Some(min_width) = self.theme.uint("min-width", &selector) {
                constraint.set_min_width(min_width as f64);
            }

            if let Some(min_height) = self.theme.uint("min-height", &selector) {
                constraint.set_min_height(min_height as f64);
            }

            if let Some(max_width) = self.theme.uint("max-width", &selector) {
                constraint.set_max_width(max_width as f64);
            }

            if let Some(max_height) = self.theme.uint("max-height", &selector) {
                constraint.set_max_height(max_height as f64);
            }

            self.set::<Constraint>("constraint", constraint);
        }

        if self.has::<f64>("spacing") {
            if let Some(spacing) = self.theme.uint("spacing", &selector) {
                self.set::<f64>("spacing", spacing.into());
            }
        }

        self.get_mut::<Selector>("selector").set_dirty(true);
    }

    pub fn update_font_properties_by_theme(&mut self, selector: &Selector) {
        if self.has::<f64>("font_size") {
            if let Some(size) = self.theme.uint("font-size", selector) {
                self.set::<f64>("font_size", f64::from(size));
            }
        }

        if self.has::<String>("font_family") {
            if let Some(font_family) = self.theme.string("font-family", selector) {
                self.set::<String>("font_family", font_family);
            }
        }

        if self.has::<Brush>("icon_brush") {
            if let Some(color) = self.theme.brush("icon-color", selector) {
                self.set::<Brush>("icon_brush", color);
            }
        }

        if self.has::<f64>("icon_size") {
            if let Some(size) = self.theme.uint("icon-size", selector) {
                self.set::<f64>("icon_size", f64::from(size));
            }
        }

        if self.has::<String>("icon_family") {
            if let Some(font_family) = self.theme.string("icon-family", selector) {
                self.set::<String>("icon_family", font_family);
            }
        }
    }
}
