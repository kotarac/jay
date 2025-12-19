use {
    crate::{
        it::{test_error::TestError, testrun::TestRun},
        tree::{Node, ToplevelNodeBase},
    },
    std::rc::Rc,
};

testcase!();

async fn test(run: Rc<TestRun>) -> Result<(), TestError> {
    run.state.ui_drag_enabled.set(true);
    let setup = run.create_default_setup().await?;
    let output = &setup.output;
    let ws = output.workspace.get().unwrap();

    // Default is true
    tassert_eq!(run.cfg.get_show_titles()?, true);

    let client = run.create_client().await?;
    let win1 = client.create_window().await?;
    win1.map().await?;
    client.sync().await;

    let container = ws.container.get().unwrap();
    let theme_title_height = run.state.theme.title_height();

    // Default -> Title shown
    tassert_eq!(container.current_title_height(), theme_title_height);

    // Global show-titles = false -> Title hidden
    run.cfg.set_show_titles(false)?;
    run.sync().await;
    tassert_eq!(container.current_title_height(), 0);

    // Global show-titles = true -> Title shown
    run.cfg.set_show_titles(true)?;
    run.sync().await;
    tassert_eq!(container.current_title_height(), theme_title_height);

    run.cfg.toggle_show_titles()?;
    run.sync().await;
    tassert_eq!(run.cfg.get_show_titles()?, false);
    tassert_eq!(container.current_title_height(), 0);

    // Global show-titles = true
    run.cfg.set_show_titles(true)?;
    run.sync().await;
    tassert_eq!(container.current_title_height(), theme_title_height);

    // Per-window hide-title override
    run.cfg.set_title_visible(setup.seat.id(), false)?;
    run.sync().await;
    tassert_eq!(container.current_title_height(), 0);

    // Per-window show-title override
    run.cfg.set_title_visible(setup.seat.id(), true)?;
    run.sync().await;
    tassert_eq!(container.current_title_height(), theme_title_height);

    // Per-window toggle-title override
    run.cfg.toggle_title(setup.seat.id())?;
    run.sync().await;
    tassert_eq!(run.cfg.get_title_visible(setup.seat.id())?, false);
    tassert_eq!(container.current_title_height(), 0);

    run.cfg.toggle_title(setup.seat.id())?;
    run.sync().await;
    tassert_eq!(run.cfg.get_title_visible(setup.seat.id())?, true);
    tassert_eq!(container.current_title_height(), theme_title_height);

    // Case: Global false, Per-window true override
    run.cfg.set_show_titles(false)?;
    run.cfg.set_title_visible(setup.seat.id(), true)?;
    run.sync().await;
    tassert_eq!(container.current_title_height(), theme_title_height);

    // Case: Global false, Per-window no override (defaults to global false)
    let win2 = client.create_window().await?;
    win2.map().await?;
    client.sync().await;
    // win2 is focused now and doesn't have an override.
    // win1's title height contribution is still theme_title_height because of its override.
    // win2 is focused, but the container is not in mono mode.
    // current_title_height() returns the maximum title height of all children.
    // Let's isolate win2 by using mono mode.
    run.cfg.set_mono(setup.seat.id(), true)?;
    run.sync().await;
    tassert_eq!(container.current_title_height(), 0);

    // Case: Vertical split layout with mixed overrides
    run.cfg.set_show_titles(true)?;
    run.cfg.set_mono(setup.seat.id(), false)?;
    run.cfg
        .set_split(setup.seat.id(), jay_config::Axis::Vertical)?;
    run.sync().await;
    // win2 is focused. Hide its title.
    run.cfg.set_title_visible(setup.seat.id(), false)?;
    run.sync().await;

    let c = ws.container.get().unwrap();
    let win2_node = c
        .children
        .iter()
        .find(|c| c.node.tl_data().node_id == win2.tl.server.tl_data().node_id)
        .unwrap();
    tassert_eq!(win2_node.title_rect.get().height(), 0);

    let win1_node = c
        .children
        .iter()
        .find(|c| c.node.tl_data().node_id == win1.tl.server.tl_data().node_id)
        .unwrap();
    tassert_eq!(win1_node.title_rect.get().height(), theme_title_height);

    // Case: Floating window with override
    run.cfg.set_floating(setup.seat.id(), true)?;
    run.sync().await;
    // win2 is floating now.
    let float_node = run
        .state
        .root
        .stacked
        .iter()
        .find_map(|n| Rc::clone(&n).node_into_float())
        .unwrap();
    tassert_eq!(float_node.title_rect.get().height(), 0);

    run.cfg.set_title_visible(setup.seat.id(), true)?;
    run.sync().await;
    tassert_eq!(float_node.title_rect.get().height(), theme_title_height);

    // Case: Mono mode ignores overrides
    run.cfg.set_floating(setup.seat.id(), false)?;
    run.cfg.set_mono(setup.seat.id(), true)?;
    run.cfg.set_show_titles(true)?;
    // Even if we hide the title for the active window, the tab bar should stay visible in mono mode
    run.cfg.set_title_visible(setup.seat.id(), false)?;
    run.sync().await;
    tassert_eq!(container.current_title_height(), theme_title_height);

    // If global is false, tab bar is hidden even if window has a show-title override
    run.cfg.set_show_titles(false)?;
    run.cfg.set_title_visible(setup.seat.id(), true)?;
    run.sync().await;
    tassert_eq!(container.current_title_height(), 0);

    // Case: Horizontal split with mixed title visibility.
    // current_title_height should return the max title height of all children.
    run.cfg.set_show_titles(true)?;
    run.cfg.set_mono(setup.seat.id(), false)?;
    run.cfg
        .set_split(setup.seat.id(), jay_config::Axis::Horizontal)?;
    run.sync().await;

    // Hide title for first window (win1), show for second (win2)
    run.cfg
        .focus(setup.seat.id(), jay_config::Direction::Left)?;
    run.cfg.set_title_visible(setup.seat.id(), false)?;
    run.cfg
        .focus(setup.seat.id(), jay_config::Direction::Right)?;
    run.cfg.set_title_visible(setup.seat.id(), true)?;
    run.sync().await;

    // The container title height should be the max height among children.
    tassert_eq!(container.current_title_height(), theme_title_height);

    let win1_node = container
        .children
        .iter()
        .find(|c| c.node.tl_data().node_id == win1.tl.server.tl_data().node_id)
        .unwrap();
    tassert_eq!(win1_node.title_rect.get().height(), 0);

    let win2_node = container
        .children
        .iter()
        .find(|c| c.node.tl_data().node_id == win2.tl.server.tl_data().node_id)
        .unwrap();
    tassert_eq!(win2_node.title_rect.get().height(), theme_title_height);
    tassert_eq!(container.children.iter().count(), 2);

    // Case: Drop window onto title bar of horizontal split
    // Grab win2 by its title bar and drop it onto win1's title bar
    let win2_title_rect = win2_node.title_rect.get();
    let win1_title_rect = win1_node.title_rect.get();
    let abs_x = container.abs_x1.get();
    let abs_y = container.abs_y1.get();

    // 1. Move to win2 title bar and press button
    setup.move_to(
        abs_x + win2_title_rect.center().0,
        abs_y + win2_title_rect.center().1,
    );
    let click = setup.mouse.click(crate::ifs::wl_seat::BTN_LEFT);
    run.sync().await;

    // 2. Move to win1 title bar
    // We need to move enough to trigger the drag threshold (10 pixels squared = 100).
    // We move to the left side of win1's title to ensure we fall into the "before win1" zone.
    setup.move_to(
        abs_x + win1_title_rect.x1() + 5,
        abs_y + win1_title_rect.center().1,
    );
    run.sync().await;

    // Trigger more movements just in case the threshold logic is sensitive to multiple events
    setup.move_to(
        abs_x + win1_title_rect.x1() + 6,
        abs_y + win1_title_rect.center().1,
    );
    run.sync().await;

    // 3. Release button (via drop of click)
    drop(click);
    run.sync().await;

    // win2 should now be the first child (to the left of win1)
    let container = ws.container.get().unwrap();
    tassert_eq!(container.children.iter().count(), 2);
    tassert_eq!(
        container.children.first().unwrap().node.node_id(),
        win2.tl.server.node_id()
    );

    Ok(())
}
