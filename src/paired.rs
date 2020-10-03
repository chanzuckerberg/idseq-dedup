use std::error::Error;

use super::fastx;

pub struct PairedRecords<T: fastx::Record, R: Iterator<Item = Result<T, std::io::Error>>> {
    records_r1: R,
    records_r2: R,
}

impl <T: fastx::Record, R: Iterator<Item = Result<T, std::io::Error>>> PairedRecords<T, R> {
    pub fn new(records_r1: R, records_r2: R) -> Self {
        PairedRecords{ records_r1: records_r1, records_r2: records_r2 }
    }

}

impl <A: fastx::Record, T: Iterator<Item = Result<A, std::io::Error>>> Iterator for PairedRecords<A, T> {
    type Item = Result<(A, A), Box<dyn Error>>;

    fn next(&mut self) -> Option<Result<(A, A), Box<dyn Error>>> {
        match (self.records_r1.next(), self.records_r2.next()) {
            (Some(Ok(r1_record)), Some(Ok(r2_record))) => {
                if r1_record.id() == r2_record.id() {
                    Some(Ok((r1_record, r2_record)))
                } else {
                    let message = format!("read pair had different read IDs: ({}, {})", r1_record.id(), r2_record.id());
                    Some(Err(Box::new(simple_error::simple_error!(message))))
                }
            },
            (None, None) => None,
            (Some(_), None) => Some(Err(Box::new(simple_error::simple_error!("reached the end of r2 before r1")))),
            (None, Some(_)) => Some(Err(Box::new(simple_error::simple_error!("reached the end of r1 before r2")))),
            (Some(Err(err)), _) => Some(Err(Box::new(err))),
            (_, Some(Err(err))) => Some(Err(Box::new(err))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bio::io::fasta;

    #[test]
    fn test_r1_longer() {
        let record = fasta::Record::with_attrs("id_a", None, &[]);
        let records_r1 = vec![Ok(record)].into_iter();
        let records_r2 = vec![].into_iter();
        let mut paired_iterator = PairedRecords::new(records_r1, records_r2);
        let result = paired_iterator.next();

        assert_eq!(result.unwrap().err().unwrap().to_string(), "reached the end of r2 before r1");
    }

    #[test]
    fn test_r2_longer() {
        let record = fasta::Record::with_attrs("id_a", None, &[]);
        let records_r1 = vec![].into_iter();
        let records_r2 = vec![Ok(record)].into_iter();
        let mut paired_iterator = PairedRecords::new(records_r1, records_r2);
        let result = paired_iterator.next();

        assert_eq!(result.unwrap().err().unwrap().to_string(), "reached the end of r1 before r2");
    }

    #[test]
    fn test_different_ids() {
        let record_r1 = fasta::Record::with_attrs("id_a", None, &[]);
        let record_r2 = fasta::Record::with_attrs("id_b", None, &[]);
        let records_r1 = vec![Ok(record_r1)].into_iter();
        let records_r2 = vec![Ok(record_r2)].into_iter();
        let mut paired_iterator = PairedRecords::new(records_r1, records_r2);
        let result = paired_iterator.next();

        assert_eq!(result.unwrap().err().unwrap().to_string(), "read pair had different read IDs: (id_a, id_b)");
    }


    struct MockFastx<'a> {
        id: &'a str,
        seq: &'a [u8],
        broken_message: Option<&'a str>
    }

    impl <'a> fastx::Record for MockFastx<'a> {
        fn id(&self) -> &str {
            &self.id
        }

        fn seq(&self) -> &[u8] {
            &self.seq
        }

        fn check(&self) -> Result<(), &str> {
            match self.broken_message {
                Some(message) => Err(message),
                None => Ok(()),
            }
        }
    }
}
