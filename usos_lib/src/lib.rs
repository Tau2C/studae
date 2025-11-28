use url::Url;

pub use models;

pub trait UsosClient {
    fn send(&self, request: Request) -> Result<Response, Error>;
}

pub struct Usos<C>
where
    C: UsosClient,
{
    client: C,
}

impl<C> Usos<C>
where
    C: UsosClient,
{
    pub fn timetable_user(
        &self,
        start: Option<TimetableUserStart>,
        days: Option<u8>,
        fields: Option<&[&str]>,
    ) -> Result<UsosTimetableUser, Error> {
        let mut params = HashMap::new();
        if let Some(start) = start {
            params.insert("start".to_owned(), start.to_string());
        }
        if let Some(days) = days {
            params.insert("days".to_owned(), days.to_string());
        }
        if let Some(fields) = fields {
            let fields_combined = fields.map(|f| f.to_string()).join("|");
            params.insert("fields".to_owned(), fields_combined);
        }
        let request_url = format!(
            "{}/services/tt/user{}",
            self.base,
            if params.is_empty() {
                String::new()
            } else {
                serde_urlencoded::to_string(&params)?
            }
        );
        self.client.send()
    }
}
