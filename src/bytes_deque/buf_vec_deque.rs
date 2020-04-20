use bytes::Buf;
use std::collections::vec_deque;
use std::collections::VecDeque;
use std::io::IoSlice;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug)]
pub(crate) struct BufVecDeque<B: Buf> {
    deque: VecDeque<B>,
    len: usize,
}

impl<B: Buf> Default for BufVecDeque<B> {
    fn default() -> Self {
        BufVecDeque {
            deque: VecDeque::default(),
            len: 0,
        }
    }
}

impl<B: Buf, I: Into<VecDeque<B>>> From<I> for BufVecDeque<B> {
    fn from(deque: I) -> Self {
        let deque = deque.into();
        let len = deque.iter().map(Buf::remaining).sum();
        BufVecDeque { deque, len }
    }
}

impl<B: Buf> BufVecDeque<B> {
    pub fn new() -> BufVecDeque<B> {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push_back(&mut self, bytes: B) {
        self.len += bytes.remaining();
        self.deque.push_back(bytes);
    }

    pub fn pop_back(&mut self) -> Option<B> {
        match self.deque.pop_back() {
            Some(b) => {
                self.len -= b.remaining();
                Some(b)
            }
            None => None,
        }
    }

    pub fn back_mut(&mut self) -> Option<BufVecDequeBackMut<B>> {
        match self.deque.pop_back() {
            Some(back) => Some(BufVecDequeBackMut {
                deque: self,
                remaining: back.remaining(),
                back: Some(back),
            }),
            None => None,
        }
    }
}

impl<B: Buf> Buf for BufVecDeque<B> {
    fn remaining(&self) -> usize {
        self.len
    }

    fn bytes(&self) -> &[u8] {
        for b in &self.deque {
            let bytes = b.bytes();
            if !bytes.is_empty() {
                return bytes;
            }
        }
        &[]
    }

    fn bytes_vectored<'a>(&'a self, dst: &mut [IoSlice<'a>]) -> usize {
        let mut n = 0;
        for b in &self.deque {
            if n == dst.len() {
                break;
            }
            n += b.bytes_vectored(&mut dst[n..]);
        }
        n
    }

    fn advance(&mut self, mut cnt: usize) {
        assert!(self.len >= cnt);
        self.len -= cnt;

        while cnt != 0 {
            let front = self.deque.front_mut().unwrap();
            let front_remaining = front.remaining();
            if cnt < front_remaining {
                front.advance(cnt);
                break;
            }

            self.deque.pop_front().unwrap();

            cnt -= front_remaining;
        }
    }
}

impl<B: Buf> IntoIterator for BufVecDeque<B> {
    type Item = B;
    type IntoIter = vec_deque::IntoIter<B>;

    fn into_iter(self) -> Self::IntoIter {
        self.deque.into_iter()
    }
}

pub struct BufVecDequeBackMut<'a, B: Buf> {
    deque: &'a mut BufVecDeque<B>,
    back: Option<B>,
    remaining: usize,
}

impl<'a, B: Buf> Deref for BufVecDequeBackMut<'a, B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        self.back.as_ref().unwrap()
    }
}

impl<'a, B: Buf> DerefMut for BufVecDequeBackMut<'a, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.back.as_mut().unwrap()
    }
}

impl<'a, B: Buf> Drop for BufVecDequeBackMut<'a, B> {
    fn drop(&mut self) {
        let back = mem::take(&mut self.back).unwrap();
        let new_remaining = back.remaining();
        if new_remaining > self.remaining {
            self.deque.len += new_remaining - self.remaining;
        } else {
            self.deque.len -= self.remaining - new_remaining;
        }
        self.deque.deque.push_back(back);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn back_mut() {
        let mut d = BufVecDeque::<VecDeque<u8>>::new();
        d.push_back(VecDeque::from(vec![3, 4]));
        d.push_back(VecDeque::from(vec![4, 6]));
        assert_eq!(4, d.remaining());
        d.back_mut().unwrap().push_back(7);
        assert_eq!(5, d.remaining());
        d.back_mut().unwrap().pop_back();
        assert_eq!(4, d.remaining());
        d.back_mut().unwrap().pop_back();
        d.back_mut().unwrap().pop_back();
        assert_eq!(2, d.remaining());
    }

    #[test]
    fn pop_back() {
        let mut d = BufVecDeque::<VecDeque<u8>>::new();
        d.push_back(VecDeque::from(vec![3, 4]));
        d.push_back(VecDeque::from(vec![4, 6, 7]));

        d.pop_back().unwrap();
        assert_eq!(2, d.remaining());
    }
}
