use makepad_render::*;

live_register!{
    use makepad_render::shader_std::*;
    
    Splitter: {{Splitter}} {
        split_bar_size: 2.0
        bar_quad:{
            instance pressed:float;
            instance hover:float;
            
            fn pixel(self) -> vec4 {
                return mix(#2, mix(#7, #f, self.pressed), self.hover);
            }
        }
        
        default_state: {
            from: {all: Play::Forward {duration: 0.1}}
            bar_quad: {pressed: 0.0, hover: 0.0}
        }
        
        hover_state: {
            from: {
                all: Play::Forward {duration: 0.1}
                state_down: Play::Forward {duration: 0.01}
            }
            bar_quad: {
                pressed: 0.0,
                hover: [{time: 0.0, value: 1.0}],
            }
        }
        
        pressed_state: {
            from: {all: Play::Forward {duration: 0.1}}
            bar_quad: {
                pressed: [{time: 0.0, value: 1.0},{time:1.0,value:0.25}],
                hover: 1.0,
            }
        }
    }
}


#[derive(Live, LiveHook)]
pub struct Splitter {
    #[rust(Axis::Horizontal)] pub axis: Axis,
    #[rust(SplitterAlign::Weighted(0.5))] pub align: SplitterAlign,
    #[rust] rect: Rect,
    #[rust] position: f32,
    #[rust] drag_start_align: Option<SplitterAlign>,
    #[track(base=default_state)] pub animator: Animator,

    default_state: Option<LivePtr>,
    hover_state: Option<LivePtr>,
    pressed_state: Option<LivePtr>,

    layout: Layout,
    bar_quad: DrawColor,
    split_bar_size: f32,
}

impl Splitter {

    pub fn begin(&mut self, cx: &mut Cx) {
        self.rect = cx.get_turtle_rect();
        self.position = self.align.to_position(self.axis, self.rect);
        cx.begin_turtle(self.layout(), Area::Empty);
    }

    pub fn middle(&mut self, cx: &mut Cx) {
        cx.end_turtle(Area::Empty);
        match self.axis {
            Axis::Horizontal => {
                self.bar_quad.draw_abs(
                    cx,
                    Rect {
                        pos: vec2(self.rect.pos.x + self.position, self.rect.pos.y),
                        size: vec2(self.split_bar_size, self.rect.size.y),
                    },
                );
                cx.set_turtle_pos(Vec2 {
                    x: self.rect.pos.x + self.position + self.split_bar_size,
                    y: self.rect.pos.y,
                });
            }
            Axis::Vertical => {
                self.bar_quad.draw_abs(
                    cx,
                    Rect {
                        pos: vec2(self.rect.pos.x, self.rect.pos.y + self.position),
                        size: vec2(self.rect.size.x, self.split_bar_size),
                    },
                );
                cx.set_turtle_pos(Vec2 {
                    x: self.rect.pos.x,
                    y: self.rect.pos.y + self.position + self.split_bar_size,
                });
            }
        }
        cx.begin_turtle(Layout::default(), Area::Empty);
    }

    pub fn end(&mut self, cx: &mut Cx) {
        cx.end_turtle(Area::Empty);
    }

    fn layout(&self) -> Layout {
        Layout {
            walk: match self.axis {
                Axis::Horizontal => Walk::wh(Width::Fixed(self.position), Height::Filled),
                Axis::Vertical => Walk::wh(Width::Filled, Height::Fixed(self.position)),
            },
            ..self.layout
        }
    }

    pub fn axis(&self) -> Axis {
        self.axis
    }

    pub fn set_axis(&mut self, axis: Axis) {
        self.axis = axis;
    }

    pub fn align(&self) -> SplitterAlign {
        self.align
    }

    pub fn set_align(&mut self, align: SplitterAlign) {
        self.align = align;
    }

    pub fn handle_event(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        dispatch_action: &mut dyn FnMut(&mut Cx, SplitterAction),
    ) {
        self.animator_handle_event(cx, event);
        match event.hits(
            cx,
            self.bar_quad.draw_vars.area,
            HitOpt {
                margin: Some(self.margin()),
                ..HitOpt::default()
            },
        ) {
            Event::FingerHover(fe) => {
                match self.axis {
                    Axis::Horizontal => cx.set_hover_mouse_cursor(MouseCursor::ColResize),
                    Axis::Vertical => cx.set_hover_mouse_cursor(MouseCursor::RowResize),
                }
                match fe.hover_state {
                    HoverState::In => {
                        self.animate_to(cx, id!(base), self.hover_state.unwrap());
                    },
                    HoverState::Out => {
                        self.animate_to(cx, id!(base), self.default_state.unwrap());
                    },
                    _ => ()
                }
            },
            Event::FingerDown(_) => {
                self.animate_to(cx, id!(base), self.pressed_state.unwrap());
                self.drag_start_align = Some(self.align);
            }
            Event::FingerUp(fe) => {
                self.drag_start_align = None;
                if fe.is_over {
                    if fe.input_type.has_hovers() {
                        self.animate_to(cx, id!(base), self.hover_state.unwrap());
                    }
                    else {
                        self.animate_to(cx, id!(base), self.default_state.unwrap());
                    }
                }
                else {
                    self.animate_to(cx, id!(base), self.default_state.unwrap());
                }
            }
            Event::FingerMove(event) => {
                if let Some(drag_start_align) = self.drag_start_align {
                    let delta = match self.axis {
                        Axis::Horizontal => event.abs.x - event.abs_start.x,
                        Axis::Vertical => event.abs.y - event.abs_start.y,
                    };
                    let new_position =
                        drag_start_align.to_position(self.axis, self.rect) + delta;
                    self.align = match self.axis {
                        Axis::Horizontal => {
                            let center = self.rect.size.x / 2.0;
                            if new_position < center - 30.0 {
                                SplitterAlign::FromStart(new_position)
                            } else if new_position > center + 30.0 {
                                SplitterAlign::FromEnd(self.rect.size.x - new_position)
                            } else {
                                SplitterAlign::Weighted(new_position / self.rect.size.x)
                            }
                        }
                        Axis::Vertical => {
                            let center = self.rect.size.y / 2.0;
                            if new_position < center - 30.0 {
                                SplitterAlign::FromStart(new_position)
                            } else if new_position > center + 30.0 {
                                SplitterAlign::FromEnd(self.rect.size.y - new_position)
                            } else {
                                SplitterAlign::Weighted(new_position / self.rect.size.y)
                            }
                        }
                    };
                    cx.redraw_view_of(self.bar_quad.draw_vars.area);
                    dispatch_action(cx, SplitterAction::Changed{axis:self.axis, align:self.align});
                }
            }
            _ => {}
        }
    }

    fn margin(&self) -> Margin {
        match self.axis {
            Axis::Horizontal => Margin {
                l: 3.0,
                t: 0.0,
                r: 7.0,
                b: 0.0,
            },
            Axis::Vertical => Margin {
                l: 0.0,
                t: 3.0,
                r: 0.0,
                b: 7.0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SplitterAlign {
    FromStart(f32),
    FromEnd(f32),
    Weighted(f32),
}

impl SplitterAlign {
    fn to_position(self, axis: Axis, rect: Rect) -> f32 {
        match axis {
            Axis::Horizontal => match self {
                Self::FromStart(position) => position,
                Self::FromEnd(position) => rect.size.x - position,
                Self::Weighted(weight) => weight * rect.size.x,
            },
            Axis::Vertical => match self {
                Self::FromStart(position) => position,
                Self::FromEnd(position) => rect.size.y - position,
                Self::Weighted(weight) => weight * rect.size.y,
            },
        }
    }
}

pub enum SplitterAction {
    Changed{axis:Axis, align:SplitterAlign},
}