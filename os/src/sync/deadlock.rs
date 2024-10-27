use alloc::vec;
use alloc::vec::Vec;

/// Banker alogrith
pub struct Banker {
    valid: bool,
    available: Vec<usize>,
    allocation: Vec<Vec<usize>>,
    need: Vec<Vec<usize>>,
}

impl Banker {
    /// create new Banker
    pub fn new() -> Self {
        Self {
            valid: false,
            available: Vec::new(),
            allocation: vec![Vec::new(); 1], // every process has a default thread 0
            need: vec![Vec::new(); 1],
        }
    }

    fn check(&self, tid: usize, res_id: usize, info: &str) {
        info!("{} check!", info);
        assert!(res_id < self.available.len());
        assert!(tid < self.allocation.len());
        assert!(tid < self.need.len());
        assert_eq!(self.need[tid].len(), self.available.len());
        assert_eq!(self.allocation[tid].len(), self.available.len());
        info!("{} check succeed", info);
    }

    /// set valid
    pub fn set_valid(&mut self, valid: bool) {
        self.valid = valid;
    }

    /// resize of thread
    pub fn resize(&mut self, tid: usize) {
        if !self.valid {
            return;
        }

        let res_len = self.available.len();
        while self.allocation.len() <= tid {
            self.allocation.push(vec![0; res_len]);
        }
        self.allocation[tid] = vec![0; res_len];

        while self.need.len() <= tid {
            self.need.push(vec![0; res_len]);
        }
        self.need[tid] = vec![0; res_len];

        info!("resize: tid: {}, res_len is {}", tid, res_len);

        for tid in 0..self.allocation.len() {
            if res_len > 0 {
                self.check(tid, res_len - 1, "resize");
            }
        }
    }

    /// add resource
    pub fn add_resource(&mut self, res_id: usize, res_val: usize) {
        if !self.valid {
            return;
        }

        while self.available.len() <= res_id {
            self.available.push(0);
        }
        self.available[res_id] = res_val;

        for allocation in &mut self.allocation {
            while allocation.len() <= res_id {
                allocation.push(0);
            }
            allocation[res_id] = 0;
        }

        for need in &mut self.need {
            while need.len() <= res_id {
                need.push(0);
            }
            need[res_id] = 0;
        }

        info!("add resource: res_id: {}, res_val: {}", res_id, res_val);

        for tid in 0..self.allocation.len() {
            self.check(tid, res_id, "add_resource");
        }
    }

    /// check deadlock
    pub fn can_alloc(&mut self, tid: usize, res_id: usize, req: usize) -> bool {
        if !self.valid {
            return true;
        }
        self.check(tid, res_id, "can_alloc");

        self.need[tid][res_id] += req;

        let res_len = self.available.len();
        let thr_len = self.allocation.len();

        let mut work = self.available.clone();
        let mut finish = vec![false; thr_len];

        info!("available is {:?}", self.available);
        info!("allocation is {:?}", self.allocation);
        info!("need is {:?}", self.need);

        loop {
            // find a thread that not finished and meet its need
            let chosen = (0..thr_len)
                .find(|&i| !finish[i] && (0..res_len).all(|j| self.need[i][j] <= work[j]));
            if let Some(tid) = chosen {
                // add the allocation to work
                for j in 0..res_len {
                    work[j] += self.allocation[tid][j];
                }
                finish[tid] = true;
            } else {
                // if no threads can be finished
                break;
            }
        }

        let res = finish.iter().all(|&x| x);

        info!("can alloc result is {}", res);
        info!("");

        res
    }

    /// alloc resource
    pub fn alloc(&mut self, tid: usize, res_id: usize, req: usize) {
        if !self.valid {
            return;
        }
        self.check(tid, res_id, "alloc");

        self.available[res_id] -= req;
        self.allocation[tid][res_id] += req;
        self.need[tid][res_id] -= req;
    }

    /// dealloc resource
    pub fn dealloc(&mut self, tid: usize, res_id: usize, req: usize) {
        if !self.valid {
            return;
        }
        self.check(tid, res_id, "dealloc");

        self.available[res_id] += req;
        self.allocation[tid][res_id] -= req;
    }
}
