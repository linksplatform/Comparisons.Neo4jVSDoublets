use {
    crate::Benched,
    std::ops::{Deref, DerefMut, Drop},
};

pub struct Fork<'f, B: Benched>(pub(crate) &'f mut B);

impl<B: Benched> Deref for Fork<'_, B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<B: Benched> DerefMut for Fork<'_, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<B: Benched> Drop for Fork<'_, B> {
    fn drop(&mut self) {
        let _ = unsafe { self.unfork() };
    }
}
