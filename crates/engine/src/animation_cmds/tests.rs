use super::*;
use photocraft_doc::LayerId;
use serde_json::json;

/// A 16×12 document with `names` as layers, bottom to top (the new-document Background is first).
fn session(names: &[&str]) -> Session {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 16, "height": 12})).unwrap();
    for n in names {
        s.execute("layer.new.layer", json!({"name": n})).unwrap();
    }
    s
}

fn doc(s: &Session) -> &Document {
    &s.active().unwrap().doc
}

fn shown(s: &Session) -> Vec<(String, bool)> {
    doc(s).layers.iter().map(|l| (l.name.clone(), l.visible)).collect()
}

fn delays(s: &Session) -> Vec<u32> {
    doc(s).frame_animation.as_ref().unwrap().frames.iter().map(|f| f.delay_ms).collect()
}

fn current(s: &Session) -> usize {
    doc(s).frame_animation.as_ref().unwrap().current
}

fn id_of(s: &Session, name: &str) -> LayerId {
    doc(s).layers.iter().find(|l| l.name == name).unwrap().id
}

#[test]
fn make_frames_from_layers_shows_one_layer_per_frame() {
    let mut s = session(&["A", "B"]);
    let r = s.execute("animation.makeFramesFromLayers", json!({"seconds": 0.2})).unwrap();
    assert_eq!(r["animation"]["frames"].as_array().unwrap().len(), 3);
    assert_eq!(delays(&s), [200, 200, 200]);
    assert_eq!(shown(&s), [("Background".into(), true), ("A".into(), false), ("B".into(), false)]);
    s.execute("animation.selectFrame", json!({"frame": 2})).unwrap();
    assert_eq!(shown(&s), [("Background".into(), false), ("A".into(), false), ("B".into(), true)]);
    assert_eq!(current(&s), 2);
}

#[test]
fn visibility_edits_change_only_the_selected_frame() {
    let mut s = session(&["A"]);
    s.execute("animation.create", json!({})).unwrap();
    s.execute("animation.newFrame", json!({})).unwrap();
    assert_eq!(current(&s), 1);
    let a = id_of(&s, "A");
    s.execute("layer.hideLayers", json!({"layer": a.0})).unwrap();
    let anim = doc(&s).frame_animation.as_ref().unwrap();
    assert!(anim.frames[0].hidden.is_empty());
    assert_eq!(anim.frames[1].hidden, [a]);
    s.execute("animation.selectFrame", json!({"frame": 0})).unwrap();
    assert!(doc(&s).layer(a).unwrap().visible);
    s.execute("animation.selectFrame", json!({"frame": 1})).unwrap();
    assert!(!doc(&s).layer(a).unwrap().visible);
}

#[test]
fn a_new_layer_shows_in_every_frame() {
    let mut s = session(&[]);
    s.execute("animation.create", json!({})).unwrap();
    s.execute("animation.newFrame", json!({})).unwrap();
    s.execute("layer.new.layer", json!({"name": "Late"})).unwrap();
    for f in [0, 1] {
        s.execute("animation.selectFrame", json!({"frame": f})).unwrap();
        assert!(shown(&s).iter().any(|(n, v)| n == "Late" && *v), "frame {f}");
    }
}

#[test]
fn delays_apply_to_the_selected_frame_a_list_or_all() {
    let mut s = session(&["A", "B"]);
    s.execute("animation.makeFramesFromLayers", json!({})).unwrap();
    s.execute("animation.setDelay", json!({"seconds": 0.5})).unwrap();
    assert_eq!(delays(&s), [500, 0, 0]);
    s.execute("animation.setDelay", json!({"delayMs": 40, "frames": [1, 2]})).unwrap();
    assert_eq!(delays(&s), [500, 40, 40]);
    s.execute("animation.setDelay", json!({"seconds": 0.1, "frames": "all"})).unwrap();
    assert_eq!(delays(&s), [100, 100, 100]);
}

#[test]
fn frame_changes_undo_and_redo() {
    let mut s = session(&["A"]);
    s.execute("animation.create", json!({})).unwrap();
    s.execute("animation.newFrame", json!({})).unwrap();
    s.execute("animation.setDelay", json!({"seconds": 1})).unwrap();
    assert_eq!(delays(&s), [0, 1000]);
    assert!(s.undo());
    assert_eq!(delays(&s), [0, 0]);
    assert!(s.undo());
    assert_eq!(delays(&s), [0]);
    assert!(s.redo());
    assert!(s.redo());
    assert_eq!(delays(&s), [0, 1000]);
}

#[test]
fn selecting_a_frame_is_not_a_history_step_and_keeps_a_clean_document_clean() {
    let mut s = session(&["A"]);
    s.execute("animation.makeFramesFromLayers", json!({})).unwrap();
    let steps = s.active().unwrap().history.can_undo();
    s.execute("animation.selectFrame", json!({"frame": 1})).unwrap();
    s.execute("animation.nextFrame", json!({})).unwrap();
    s.execute("animation.nextFrame", json!({})).unwrap();
    assert_eq!(current(&s), 1, "next stops at the last frame");
    s.execute("animation.previousFrame", json!({})).unwrap();
    s.execute("animation.previousFrame", json!({})).unwrap();
    assert_eq!(current(&s), 0);
    assert_eq!(s.active().unwrap().history.can_undo(), steps);
}

#[test]
fn loop_counts() {
    let mut s = session(&[]);
    s.execute("animation.create", json!({})).unwrap();
    for (v, plays) in [(json!("once"), 1), (json!(5), 5), (json!("forever"), 0), (json!(1), 1), (json!(0), 0)] {
        let r = s.execute("animation.setLoop", json!({"loops": v})).unwrap();
        assert_eq!(doc(&s).frame_animation.as_ref().unwrap().loops.plays(), plays, "{r}");
    }
    assert!(s.execute("animation.setLoop", json!({"loops": "sometimes"})).is_err());
    assert!(s.execute("animation.setLoop", json!({})).is_err());
}

#[test]
fn move_reverse_and_delete_keep_the_selection_on_its_frame() {
    let mut s = session(&["A", "B"]);
    s.execute("animation.makeFramesFromLayers", json!({})).unwrap();
    for (i, ms) in [(0, 10), (1, 20), (2, 30)] {
        s.execute("animation.setDelay", json!({"delayMs": ms, "frames": [i]})).unwrap();
    }
    s.execute("animation.selectFrame", json!({"frame": 0})).unwrap();
    s.execute("animation.moveFrame", json!({"from": 0, "to": 2})).unwrap();
    assert_eq!(delays(&s), [20, 30, 10]);
    assert_eq!(current(&s), 2, "the selected frame moved with it");
    s.execute("animation.reverseFrames", json!({})).unwrap();
    assert_eq!(delays(&s), [10, 30, 20]);
    assert_eq!(current(&s), 0);
    s.execute("animation.deleteFrame", json!({"frame": 1})).unwrap();
    assert_eq!(delays(&s), [10, 20]);
    assert_eq!(current(&s), 0);
    s.execute("animation.deleteFrame", json!({})).unwrap();
    assert_eq!(delays(&s), [20]);
    assert!(s.execute("animation.deleteFrame", json!({})).is_err(), "the last frame stays");
}

#[test]
fn deleting_the_animation_keeps_the_layers_as_they_show() {
    let mut s = session(&["A"]);
    s.execute("animation.makeFramesFromLayers", json!({})).unwrap();
    s.execute("animation.selectFrame", json!({"frame": 1})).unwrap();
    let before = shown(&s);
    s.execute("animation.delete", json!({})).unwrap();
    assert!(doc(&s).frame_animation.is_none());
    assert_eq!(shown(&s), before);
    assert!(s.execute("animation.newFrame", json!({})).is_err());
}

#[test]
fn bad_params_are_errors_not_panics() {
    let mut s = session(&["A"]);
    assert!(s.execute("animation.selectFrame", json!({"frame": 0})).is_err(), "no animation yet");
    s.execute("animation.create", json!({})).unwrap();
    assert!(s.execute("animation.create", json!({})).is_err(), "only one");
    for (cmd, p) in [
        ("animation.selectFrame", json!({"frame": 9})),
        ("animation.selectFrame", json!({})),
        ("animation.selectFrame", json!({"frame": -1})),
        ("animation.setDelay", json!({})),
        ("animation.setDelay", json!({"seconds": -1})),
        ("animation.setDelay", json!({"seconds": 1e12})),
        ("animation.setDelay", json!({"seconds": "soon"})),
        ("animation.setDelay", json!({"seconds": 1, "frames": [7]})),
        ("animation.setDelay", json!({"seconds": 1, "frames": 3})),
        ("animation.moveFrame", json!({"from": 0, "to": 5})),
        ("animation.deleteFrame", json!({"frame": 4})),
    ] {
        // A refused command leaves the animation, the layers and the history exactly as they were.
        let snapshot = |s: &Session| {
            let st = s.active().unwrap();
            (st.doc.frame_animation.clone(), shown(s), st.revision, st.history.can_undo())
        };
        let before = snapshot(&s);
        assert!(s.execute(cmd, p.clone()).is_err(), "{cmd} {p}");
        assert_eq!(snapshot(&s), before, "{cmd} {p} changed the document");
    }
}

#[test]
fn a_stored_hidden_list_in_any_order_still_hides_the_right_layers() {
    let mut s = session(&["A", "B", "C"]);
    s.execute("animation.makeFramesFromLayers", json!({})).unwrap();
    let ids: Vec<_> = doc(&s).layers.iter().map(|l| l.id).collect();
    // A hand-edited file: frame 1 hides Background, B and C, written in descending order with a repeat.
    let a = s.active_mut().unwrap();
    let mut d = (*a.doc).clone();
    d.frame_animation.as_mut().unwrap().frames[1].hidden = vec![ids[3], ids[2], ids[0], ids[2]];
    d.frame_animation.as_mut().unwrap().clamp();
    a.doc = std::sync::Arc::new(d);
    s.execute("animation.selectFrame", json!({"frame": 1})).unwrap();
    assert_eq!(shown(&s), [("Background".into(), false), ("A".into(), true), ("B".into(), false), ("C".into(), false)]);
    assert_eq!(doc(&s).frame_animation.as_ref().unwrap().frames[1].hidden, [ids[0], ids[2], ids[3]], "sorted and without repeats");
}

#[test]
fn frames_from_layers_keep_the_eye_of_layers_inside_a_group() {
    use photocraft_doc::{Group, Layer, LayerContent};
    let mut s = session(&["A"]);
    // A group holding one visible and one hidden child, above the others.
    let mut hidden_child = Layer::new("hidden child", LayerContent::Group(Group { children: Vec::new(), expanded: true, artboard: None }));
    hidden_child.visible = false;
    let shown_child = Layer::new("shown child", LayerContent::Group(Group { children: Vec::new(), expanded: true, artboard: None }));
    let (hid, shown_id) = (hidden_child.id, shown_child.id);
    let group = Layer::new("G", LayerContent::Group(Group { children: vec![shown_child, hidden_child], expanded: true, artboard: None }));
    let st = s.active_mut().unwrap();
    let mut d = (*st.doc).clone();
    d.layers.push(group);
    st.doc = std::sync::Arc::new(d);
    s.execute("animation.makeFramesFromLayers", json!({})).unwrap();
    let last = doc(&s).layers.len() - 1;
    s.execute("animation.selectFrame", json!({"frame": last})).unwrap();
    let g = doc(&s).layers.last().unwrap();
    let kids = g.children().unwrap();
    assert!(g.visible);
    assert_eq!(kids.iter().map(|k| (k.id, k.visible)).collect::<Vec<_>>(), [(shown_id, true), (hid, false)], "the hidden child stays hidden");
    s.execute("animation.selectFrame", json!({"frame": 0})).unwrap();
    s.execute("animation.selectFrame", json!({"frame": last})).unwrap();
    assert!(!doc(&s).layers.last().unwrap().children().unwrap()[1].visible);
}

#[test]
fn navigation_replies_are_compact_and_not_journaled() {
    let mut s = session(&["A", "B"]);
    s.execute("animation.makeFramesFromLayers", json!({})).unwrap();
    let r = s.execute("animation.nextFrame", json!({})).unwrap();
    assert_eq!(r, json!({"animation": {"current": 1, "frameCount": 3}}));
    for id in ["animation.selectFrame", "animation.nextFrame", "animation.previousFrame"] {
        assert!(!crate::commands::find(id).unwrap().journal, "{id} is navigation, not a recorded step");
    }
}

#[test]
fn the_delay_limit_in_the_error_is_exact() {
    let mut s = session(&["A"]);
    s.execute("animation.create", json!({})).unwrap();
    let e = s.execute("animation.setDelay", json!({"seconds": 1000})).unwrap_err().to_string();
    assert!(e.contains("655.35"), "{e}");
}

#[test]
fn too_many_layers_for_frames_are_refused() {
    use photocraft_doc::{Group, Layer, LayerContent};
    let mut s = session(&[]);
    let st = s.active_mut().unwrap();
    let mut d = (*st.doc).clone();
    for i in 0..1001 {
        d.layers.push(Layer::new(format!("L{i}"), LayerContent::Group(Group { children: Vec::new(), expanded: true, artboard: None })));
    }
    st.doc = std::sync::Arc::new(d);
    assert!(s.execute("animation.makeFramesFromLayers", json!({})).is_err());
    assert!(doc(&s).frame_animation.is_none());
}

#[test]
fn selecting_a_frame_waits_for_a_background_job_like_an_edit_does() {
    let mut s = session(&["A", "B"]);
    s.execute("animation.makeFramesFromLayers", json!({})).unwrap();
    let job = match s.start("filter.blur.gaussianBlur", json!({"radius": 2})).unwrap() {
        crate::jobs::Started::Job(id) => id,
        crate::jobs::Started::Done(_) => return, // ran inline (no threads): nothing to wait for
    };
    let e = s.execute("animation.selectFrame", json!({"frame": 1})).unwrap_err().to_string();
    assert!(e.contains("still running"), "{e}");
    assert_eq!(current(&s), 0);
    let t = std::time::Instant::now();
    while !s.poll_jobs().iter().any(|e| e.id == job) {
        assert!(t.elapsed() < std::time::Duration::from_secs(60), "the job did not finish");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    s.execute("animation.selectFrame", json!({"frame": 1})).unwrap();
    assert_eq!(current(&s), 1);
}
