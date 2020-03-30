use std::io::{self, Read};
use std::process;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

// This day is a great example of my approach to these challenges and how it differs from
// some others'. Where possible, I like to build extensible representations of the input data
// that (a) make it really easy to understand how the problem is being solved and (b) could 
// potentially be used to answer questions beyond the two we happen to have been given.
//
// Compare mine to AxlLind (https://github.com/AxlLind/AdventOfCode2019/blob/master/src/bin/06.rs).
// He's built a HashMap of parents to children.  You can use that in cunning ways to traverse
// the structure, and indeed he does, but (a) it's a lot harder to wrap your head around how
// and why his works, and (b) his executes about 50% slower than mine because he hasn't built an
// actual hierarchical relationship and so has to iterate through his HashMap for each level of
// traversal. The price I've paid for these gains is that there's _way_ more code here, and I
// imagine it took me a lot longer!

fn main() {
    let start_time = std::time::Instant::now();

    // Load from file and construct a tree and map of OrbitalObjects.
    let map = load_orbits("day6/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        process::exit(1);
    });

    // Day 6 part 1:
    // The total depth is simply the individual depth of each orbital object, summed.
    let total_depth: u32 = map.values().map(|rc| rc.borrow_mut().get_depth()).sum();

    // Day 6 part 2:
    // The distance Santa needs to move to orbit the same object as us is the distance between the
    // object he's orbiting and our common ancestor, plus the distance between the object we're
    // orbiting and our common ancestor.  Find all three objects.  That calculation can be
    // expressed as (Santa's depth plus our depth minus twice the ancestor's depth minus 2).
    let santa = map.get("SAN").unwrap_or_else(|| {
        println!("Invalid input file - SAN not found!");
        process::exit(1);
    });
    let you = map.get("YOU").unwrap_or_else(|| {
        println!("Invalid input file - YOU not found!");
        process::exit(1);
    });
    let common_ancestor = OrbitalObject::find_common_ancestor(santa, you).unwrap_or_else(|| {
        println!("Invalid input file - YOU and SAN aren't indirectly orbiting a common object!");
        process::exit(1);
    });

    let distance = santa.borrow().depth.unwrap() + you.borrow().depth.unwrap()
        - 2
        - (2 * common_ancestor.borrow().depth.unwrap());

    println!(
        "Total depth: {}\nDistance: {}\nTime: {}ms",
        total_depth,
        distance,
        start_time.elapsed().as_millis()
    );
}

/// An object that may be in orbit around another object and/or may have other objects in orbit
/// around it.
pub struct OrbitalObject {
    /// The name of this object.
    name: String,

    /// The number of objects this object directly or indirectly orbits.  That is, an object
    /// orbiting nothing will have a depth of 0. An object orbiting another object that itself
    /// is orbiting nothing will have a depth of 1. An object orbiting _that_ object will have
    /// a depth of 2, etc.
    depth: Option<u32>,

    /// The object that this object orbits.
    parent: Option<Weak<RefCell<Self>>>,

    /// Objects in orbit around this object.
    children: Vec<Rc<RefCell<Self>>>,
}

impl OrbitalObject {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            depth: None,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Links parent and child `OrbitalObject`s together (bidirectionally).
    ///
    /// # Examples
    ///
    /// ```
    /// let parent = Rc::new(RefCell::new(OrbitalObject::new(parent_name)));
    /// let child = Rc::new(RefCell::new(OrbitalObject::new(child_name)));
    /// OrbitalObject::associate(&parent, &child);
    /// ```
    pub fn associate(parent: &Rc<RefCell<Self>>, child: &Rc<RefCell<Self>>) {
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        parent.borrow_mut().children.push(Rc::clone(child));
    }

    /// Returns the orbital depth of this object, including calculating it (by following the
    /// chain of ancestors) if it hasn't yet been determined.
    ///
    /// Do not call this until the tree of objects has been fully constructed.
    pub fn get_depth(&mut self) -> u32 {
        // Return current value if already calculated, else calculate it - parent's depth plus
        // one, or zero if we have no parent.
        if self.depth.is_none() {
            self.depth = Some(self.parent.as_ref().map_or(0, |parent| {
                parent
                    .upgrade()
                    .map_or(0, |parent| parent.borrow_mut().get_depth() + 1)
            }));
        }
        self.depth.unwrap()
    }

    /// Locates the lowest-level object that `first` and `second` both directly or indirectly
    /// orbit. Returns `None` if no such common ancestor exists.
    ///
    /// # Examples
    ///
    /// ```
    /// let parent = Rc::new(RefCell::new(OrbitalObject::new(parent_name)));
    /// let child_one = Rc::new(RefCell::new(OrbitalObject::new(child_one_name)));
    /// let child_two = Rc::new(RefCell::new(OrbitalObject::new(child_two_name)));
    /// OrbitalObject::associate(&parent, &child_one);
    /// OrbitalObject::associate(&parent, &child_two);
    ///
    /// let ancestor = OrbitalObject::find_common_ancestor(&child_one, &child_two);
    /// assert_eq!(ancestor, parent);
    /// ```
    #[must_use]
    pub fn find_common_ancestor(
        first: &Rc<RefCell<Self>>,
        second: &Rc<RefCell<Self>>,
    ) -> Option<Rc<RefCell<Self>>> {
        // Find the first of `first`'s ancestors that's in the set of `second`'s ancestors.
        let first_object = first.borrow();
        let second_ancestors = second.borrow().get_ancestors();

        if second_ancestors.contains(&first_object.name) {
            Some(first.clone())
        } else {
            first_object.parent.as_ref().and_then(|weakref| {
                weakref
                    .upgrade()
                    .and_then(|parent| OrbitalObject::find_common_ancestor(&parent, second))
            })
        }
    }

    // Construct a set of all ancestors of this object, i.e. everything it directly or
    // indirectly orbits, plus itself.
    fn get_ancestors(&self) -> HashSet<String> {
        let mut set = self.parent.as_ref().map_or(HashSet::new(), |parent_ref| {
            parent_ref
                .upgrade()
                .map_or(HashSet::new(), |parent| parent.borrow().get_ancestors())
        });

        set.insert(self.name.clone());
        set
    }
}

/// Loads a definition of orbits from file and constructs a tree of `OrbitalObject`s from them, plus
/// a map indexed by `OrbitalObject.name`.  Returns an error if the file cannot be found or loaded,
/// or if its contents are invalid.
///
/// # Errors
///
/// Returns an `io::Error` for any issues reading `source_file`, including invalid contents.
pub fn load_orbits(
    source_file: &str,
) -> Result<HashMap<String, Rc<RefCell<OrbitalObject>>>, io::Error> {
    let mut input = std::fs::File::open(source_file)?;
    let mut orbits = String::new();
    input.read_to_string(&mut orbits)?;

    let mut map: HashMap<String, Rc<RefCell<OrbitalObject>>> = HashMap::new();

    for line in orbits.lines() {
        // Lines are in the form PARENTNAME)CHILDNAME.
        let objects: Vec<&str> = line.split(')').collect();
        if objects.len() != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid input line: {}", line),
            ));
        }

        let parent = get_orbital_object(&mut map, objects[0]);
        let child = get_orbital_object(&mut map, objects[1]);

        OrbitalObject::associate(&parent, &child);
    }

    Ok(map)
}

// Find the OrbitalObject with a given name, either by locating an existing one in `map`, or
// constructing one and adding it to the map.
fn get_orbital_object(
    map: &mut HashMap<String, Rc<RefCell<OrbitalObject>>>,
    name: &str,
) -> Rc<RefCell<OrbitalObject>> {
    if let Some(rc) = map.get(name) {
        rc.clone()
    } else {
        let obj = OrbitalObject::new(String::from(name));
        let rc = Rc::new(RefCell::new(obj));
        map.insert(String::from(name), rc.clone());
        rc
    }
}
