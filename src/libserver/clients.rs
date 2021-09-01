use crate::{compute_scores, make_strengths_graph, make_weaknesses_graph, Client};
use std::{cmp::Ordering, net::SocketAddr};

pub struct Clients {
    pub p1: Option<Client>,
    pub p2: Option<Client>,
}

impl Clients {
    pub fn default() -> Clients {
        Clients { p1: None, p2: None }
    }

    pub fn add(&mut self, client: Client) {
        if self.p1.is_none() {
            self.p1 = Some(client)
        } else if self.p2.is_none() {
            self.p2 = Some(client)
        } else {
            panic!("clients is full.");
        }
    }

    pub fn set_ready(&mut self, addr: SocketAddr) {
        if let Some(ref mut p1) = self.p1 {
            p1.ready = p1.ready || p1.addr == addr;
        }

        if let Some(ref mut p2) = self.p2 {
            p2.ready = p2.ready || p2.addr == addr;
        }
    }

    pub fn both_ready(&self) -> bool {
        if let Some(ref p1) = self.p1 {
            if let Some(ref p2) = self.p2 {
                if p1.ready && p2.ready {
                    return true;
                }
            }
        }
        false
    }

    pub fn send_choices(&mut self) {
        let mut rng = rand::thread_rng();

        let p1_choices = match self.p1 {
            Some(ref mut p) => {
                p.set_choices(&mut rng);
                p.choices.as_ref().unwrap().join(",")
            }
            _ => panic!("Unset client 'p1' cannot get choices !"),
        };

        let p2_choices = match self.p2 {
            Some(ref mut p) => {
                p.set_choices(&mut rng);
                p.choices.as_ref().unwrap().join(",")
            }
            _ => panic!("Unset client 'p2' cannot get choices !"),
        };

        if let Some(ref p) = self.p1 {
            let msg = format!("yours:{};theirs:{}", p1_choices, p2_choices);
            p.tx.unbounded_send(tungstenite::Message::Text(msg))
                .unwrap();
        }

        if let Some(ref p) = self.p2 {
            let msg = format!("yours:{};theirs:{}", p2_choices, p1_choices);
            p.tx.unbounded_send(tungstenite::Message::Text(msg))
                .unwrap();
        }
    }

    pub fn set_selected(&mut self, addr: SocketAddr, type_: String) {
        if let Some(ref mut p) = self.p1 {
            if p.addr == addr {
                p.selected = Some(type_.clone());
            }
        }

        if let Some(ref mut p) = self.p2 {
            if p.addr == addr {
                p.selected = Some(type_);
            }
        }
    }

    pub fn both_selected(&self) -> bool {
        if let Some(ref p1) = self.p1 {
            if let Some(ref p2) = self.p2 {
                if p1.selected.is_some() && p2.selected.is_some() {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_selected(&self) -> Option<(String, String)> {
        let p1_selected: String;
        let p2_selected: String;
        if let Some(ref p1) = self.p1 {
            if let Some(ref p1s) = p1.selected {
                p1_selected = p1s.clone();
            } else {
                return None;
            }
        } else {
            return None;
        }

        if let Some(ref p2) = self.p2 {
            if let Some(ref p2s) = p2.selected {
                p2_selected = p2s.clone();
            } else {
                return None;
            }
        } else {
            return None;
        }

        Some((p1_selected, p2_selected))
    }

    pub fn send_outcomes(&self) {
        if let Some((p1_selected, p2_selected)) = self.get_selected() {
            let p1 = self.p1.as_ref().unwrap();
            let p2 = self.p2.as_ref().unwrap();

            let strengths = make_strengths_graph();
            let weaknesses = make_weaknesses_graph();

            let (p1_score, p2_score) = compute_scores(
                p1_selected.clone(),
                p2_selected.clone(),
                strengths,
                weaknesses,
            );

            println!("p1: {} vs p2: {}", p1_score, p2_score);

            match p1_score.cmp(&p2_score) {
                Ordering::Equal => {
                    p1.send_outcome("tie", p1_selected.clone(), p2_selected.clone());
                    p2.send_outcome("tie", p1_selected, p2_selected);
                }
                Ordering::Greater => {
                    p1.send_outcome("won", p1_selected.clone(), p2_selected.clone());
                    p2.send_outcome("lost", p2_selected, p1_selected);
                }
                Ordering::Less => {
                    p1.send_outcome("lost", p1_selected.clone(), p2_selected.clone());
                    p2.send_outcome("won", p2_selected, p1_selected);
                }
            }
        } else {
            panic!("Cannot find outcome !");
        }
    }

    pub fn send_msg(&self, addr: SocketAddr, msg: String) {
        if let Some(p1) = &self.p1 {
            if p1.addr == addr {
                p1.tx
                    .unbounded_send(tungstenite::Message::Text(msg.clone()))
                    .unwrap();
            }
        }

        if let Some(p2) = &self.p2 {
            if p2.addr == addr {
                p2.tx
                    .unbounded_send(tungstenite::Message::Text(msg))
                    .unwrap();
            }
        }
    }

    pub fn reset(&mut self) {
        if let Some(ref mut p1) = self.p1 {
            p1.choices = None;
            p1.selected = None;
            p1.ready = false;
        }
        if let Some(ref mut p2) = self.p2 {
            p2.choices = None;
            p2.selected = None;
            p2.ready = false;
        }
    }
}
