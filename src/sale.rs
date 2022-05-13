//! Ticket sale struct.

use std::cmp::Ordering;
use std::error::Error;
use std::io::Read;

use chrono::{DateTime, Utc};
use csv::{StringRecord, StringRecordsIter};
use crate::context::SalesContext;
use crate::sale::kind::{SaleKind, Seller};

pub(crate) mod kind;
pub(crate) mod price_deriving;
pub(crate) mod plus;
pub(crate) mod ambiguity;

static RECORD_LEN: usize = 13;
static NA: &str = "N/A";

fn field_or_na(o: Option<&&str>) -> Option<String> {
  if let Some(s) = o {
    if s != &NA && s.len() > 0 {
      return Some(s.to_string());
    }
  }
  return None;
}

/// A sale, as from the CSV.
#[derive(Clone, Debug)]
pub(crate) struct Sale {
  /// Sale date and time.
  pub(crate) when: DateTime<Utc>,
  /// Buyer e-mail address.
  pub(crate) buyer_email: Option<String>,
  /// Buyer username.
  pub(crate) buyer_username: Option<String>,
  /// Sale value in cents.
  pub(crate) value: usize,
  /// Seller data (online or offline).
  pub(crate) sale_kind: SaleKind,
  /// Seller name (absent when online)
  pub(crate) seller_name: Option<String>,
  /// Seller ID string (no idea where it comes from).
  pub(crate) seller_id: Option<String>,
  /// Seller email (absent when online).
  pub(crate) seller_email: Option<String>,
  /// Token (also no idea).
  pub(crate) token: String,
  /// Sale ID (what?).
  pub(crate) sale_id: String,
  /// Card name (it's all blank?).
  pub(crate) card_name: Option<String>,
  /// Card prefix.
  pub(crate) card_pfx: Option<String>,
  /// Card suffix.
  pub(crate) card_sfx: Option<String>
}

impl Sale {
  /// Compare sales' dates. Useful for Vec::sort_by and such.
  pub(crate) fn cmp_dates(&self, other: &Self) -> Ordering {
    return self.when.cmp(&other.when);
  }
  
  /// Return the "real price", after undoing fees and such.
  pub(crate) fn real_price(&self) -> usize {
    return self.sale_kind.undo_fee(self.value);
  }
}

impl TryFrom<(StringRecord, &SalesContext)> for Sale {
  type Error = Box<dyn Error>;

  fn try_from(
    (r, ctx): (StringRecord, &SalesContext)
  ) -> Result<Self, Self::Error> {
    let v: Vec<&str> = r.into_iter().collect();
    if v.len() != RECORD_LEN {
      return Err(
        format!("expected {} columns, got {}", RECORD_LEN, v.len()).into()
      );
    }
    let val: f64 = v.get(3).ok_or("f64 parse error")?.parse()?;
    return Ok(Self {
      when: DateTime::parse_from_rfc3339(v.get(0).unwrap())?.into(),
      buyer_email: field_or_na(v.get(1)),
      buyer_username: field_or_na(v.get(2)),
      value: (val * 100.0).round() as usize,
      sale_kind: {
        if v.get(4).unwrap().contains("Online") {
          SaleKind::Online(ctx.online_fee)
        } else {
          SaleKind::Offline
        }
      },
      seller_name: field_or_na(v.get(5)),
      seller_id: field_or_na(v.get(6)),
      seller_email: field_or_na(v.get(7)),
      token: v.get(8).unwrap().to_string(),
      sale_id: v.get(9).unwrap().to_string(),
      card_name: field_or_na(v.get(10)),
      card_pfx: field_or_na(v.get(11)),
      card_sfx: field_or_na(v.get(12)),
    });
  }
}

impl Sale {
  /// Returns a vec of sales, sorted by date.
  pub(crate) fn parse_csv<'r, R: Read>(
    records: StringRecordsIter<'r, R>,
    ctx: &SalesContext
  ) -> (Vec<Sale>, Vec<Box<dyn Error>>) {
    let mut sv: Vec<Sale> = Vec::new();
    let mut ev: Vec<Box<dyn Error>> = Vec::new();
    for recres in records {
      match recres {
        Ok(rec) => {
          match Sale::try_from((rec, ctx)) {
            Ok(s) => sv.push(s),
            Err(b) => ev.push(b),
          }
        },
        Err(e) => ev.push(Box::new(e)),
      }
    }
    sv.sort_by(Sale::cmp_dates);
    return (sv, ev);
  }

  /// Infer the seller, if at all possible.
  pub(crate) fn seller(&self) -> Option<Seller> {
    return match (&self.sale_kind, &self.seller_name) {
      (SaleKind::Online(_), _) => Some(Seller::Online),
      // (SaleKind::Online(_), Some(_)) => None,
      (SaleKind::Offline, None) => None,
      (SaleKind::Offline, Some(s)) => Some(Seller::Offline(s.clone())),
    };
  }
}
