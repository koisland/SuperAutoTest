//! Visualize [`Team`](crate::Team) battles/orders.
//! * Storing battle data and building digraphs allows visualization of battle logic but causes a performance hit.
//! * **~225% increase in benchmarking times**. (860 ns -> 2.7 us)
//!
//! This can be toggled off with `general.build_graphs` in `.saptest.toml`. By default, graphs are `enabled`.
//! ```toml
//! [general]
//! build_graph = false
//! ```

/// Build and format directed acrylic graphs of battle phases.
pub mod dag;
