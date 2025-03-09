use crate::ports::outputs::{verify::Verify, database::{Item, GetItem, CreateItem, DeleteItem}};
use crate::domain::types::{Verification, Phone, EmailAddress, VerificationMedia};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use reqwest::Client;
use error::Error;

mod error;


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Twilio {
    account_sid: String,
    service_sid: String,
    credentials: Credentials,
    base_url: String,
    custom_code: bool,
    #[serde(skip)]
    client: Client
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Credentials {
    #[serde(alias = "user_name")]
    username: String,
    password: String
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Response {
    valid: bool
}


impl Twilio {
    fn code(verification: Verification) -> String {
        let mut code = String::new();
        if verification.code < 100000 {
            code.push('0');
        }
        let string = verification.code.to_string();
        code.push_str(&string);
        code
    }

    async fn initiate_request(&self, form: &HashMap<&str, &str>) -> Result<(), Error> {
        let base_url = self.base_url.as_str().trim_end_matches("/");
        let url = format!("{base_url}/Services/{}/Verifications", self.service_sid);
        let (username, password) = (self.credentials.username.as_str(), Some(self.credentials.password.as_str()));
        let res = self.client.post(url).basic_auth(username, password).form(form).send().await.map_err(Error::internal)?;
        if res.status() != 201 {
            let err = format!("{:?}", res);
            Err(Error::internal(err))?
        }
        Ok(())
    }

    async fn verify(&self, form: &HashMap<&str, &str>) -> Result<(), Error> {
        let base_url = self.base_url.as_str().trim_end_matches("/");
        let url = format!("{base_url}/Services/{}/VerificationCheck", self.service_sid);
        let (username, password) = (self.credentials.username.as_str(), Some(self.credentials.password.as_str()));
        let res: Response = self.client.post(url).basic_auth(username, password).form(form).send().await.map_err(Error::internal)?.json().await.map_err(Error::internal)?;
        if !res.valid {
            return Err(Error::InvalidCode);
        }
        Ok(())
    }
}



impl Verify<Phone> for Twilio {
    type Error = Error;
    type Channel = VerificationMedia;
    type Verification = Verification;

    async fn initiate<DB: CreateItem<Self::Verification>>(&self, contact: &Phone, channel: Self::Channel, db: &DB) -> Result<(), Self::Error> {
        let number = contact.as_str();
        let channel = channel.to_string();
        let mut form = HashMap::new();
        form.insert("To", number);
        form.insert("Channel", channel.as_str());
        self.initiate_request(&form).await
    }

    async fn verify<DB: GetItem<Self::Verification> + DeleteItem<Self::Verification>>(&self, contact: &Phone,  code: &str, db: &DB) -> Result<(), Self::Error> {
        todo!()
    }
}
