use std::cell::RefCell;

mod test;

#[derive(PartialEq)]
enum PointerBox<'a> {
    ReputationProof(&'a ReputationProof<'a>),
    String(String)
}

#[derive(Clone)]
struct ReputationProof<'a> {
    box_id: Vec<u8>,
    token_id: Vec<u8>,
    total_amount: i64,
    outputs: Vec<RefCell<ReputationProof<'a>>>,
    pointer_box: Option<&'a PointerBox<'a>>,
}

impl<'a> PartialEq for ReputationProof<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.box_id == other.box_id
    }
}

impl <'a, 'b> ReputationProof<'a> {
    fn new(
        box_id: Vec<u8>,
        token_id: Vec<u8>,
        total_amount: i64,
        outputs: Vec<RefCell<ReputationProof<'a>>>,
        pointer_box: Option<&'a PointerBox<'a>>,
    ) -> ReputationProof<'a> {
        ReputationProof {
            box_id,
            token_id,
            total_amount,
            outputs,
            pointer_box,
        }
    }

    /**
        Creates a new reputation proof from scratch.
    */
    pub fn create(
        total_amount: i64,
        pointer_box: Option<&'b PointerBox<'a>>,
    ) -> ReputationProof<'a> {
        return ReputationProof::new(
            vec![], vec![],
            total_amount,  vec![],
            pointer_box
        )
    }

    /**
        Creates a new reputation proof from the current one.
        Raises exceptions if any rule is violated.
    */
    pub fn spend(&'b mut self,
                 amount: i64,
                 pointer_box: Option<&'b PointerBox<'a>>,
    ) -> RefCell<ReputationProof<'a>> {
        let new_box = RefCell::create(
                ReputationProof::new(
                    vec![], self.get_token_id(),
                    amount, vec![],
                    pointer_box
            )
        );
        self.outputs.push(new_box);
        return new_box;
    }

    /**
        Get the proportion of reputation that have the out_index output over the total.
    */
    fn expended_proportion(&self, out_index: usize) -> f64 {
        return self.outputs[out_index].into_inner().total_amount as f64 / self.total_amount as f64;
    }

    /**
        Optimize memory if the childs don't store the token_id and get it from the root.
    */
    fn get_token_id(&self) -> Vec<u8> {
        return self.token_id.clone()  // TODO
    }

    /**
        Compute the reputation of a pointer searching on all the output tree.
    */
    pub fn compute(&self, pointer: Option<&'b PointerBox<'a>>) -> f64 {
        if self.pointer_box.is_some() {
            // Recursive case: if there is pointer, uses the pointer_box's reputation.
            if pointer.is_some() && self.pointer_box == pointer {
                1.00
            } else {
                0.00 // ptr.compute(None)  // TODO
            }
        } else {
            // Base case: if there is not pointer, computes the reputation directly.
            self.outputs
                .iter()
                .enumerate()
                .map(
                    |(index, out)|
                    self.expended_proportion(index) * out.compute(pointer)
                )
                .sum()
        }
    }

}

fn static_spend<'a, 'b>
(
    main: &'b mut RefCell<ReputationProof<'a>>,
    amount: i64,
    pointer_box: Option<&'b PointerBox<'a>>
)
    -> &'b RefCell<ReputationProof<'a>>
{
    (*main).spend(amount, pointer_box)
}

fn static_compute_reputation<'a, 'b>
(
    main: &'b mut ReputationProof<'a>,
    pointer_box: &'b PointerBox<'a>
)
    -> f64
{
    (*main).compute(Some(pointer_box))
}
