use crate::ports::outputs::{database::{CreateItem, DeleteItem, GetItem, GetItems, Item}, verify::{self, Verify, Code}};
use crate::domain::types::{Verification, Phone, EmailAddress, VerificationMedia, Contact, Key};
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
    friendly_name: Option<String>,
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
    #[serde(alias = "sid")]
    id: String,
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

    async fn initiate_request(&self, form: &HashMap<&str, &str>) -> Result<Response, Error> {
        let base_url = self.base_url.as_str().trim_end_matches("/");
        let url = format!("{base_url}/Services/{}/Verifications", self.service_sid);
        let (username, password) = (self.credentials.username.as_str(), Some(self.credentials.password.as_str()));
        let res = self.client.post(url).basic_auth(username, password).form(form).send().await.map_err(Error::internal)?;
        if res.status() != 201 {
            let err = format!("{:?}", res);
            Err(Error::internal(err))?
        }
        Ok(res.json().await.map_err(Error::internal)?)
    }

    async fn verify_request(&self, form: &HashMap<&str, &str>, id: Option<&str>) -> Result<(), Error> {
        let base_url = self.base_url.as_str().trim_end_matches("/");
        let url = match id {
            None => format!("{base_url}/Services/{}/VerificationCheck", self.service_sid),
            Some(id) => format!("{base_url}/Services/{}/Verifications/{}", self.service_sid, id)
        };
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
    type Verification = Verification<String>;

    async fn initiate<DB: CreateItem<Self::Verification>>(&self, contact: &Phone, channel: Self::Channel, db: &DB) -> Result<(), Self::Error> {
        let number = contact.as_str();
        let channel = channel.to_string();
        let mut form = HashMap::new();
        if let Some(name) = &self.friendly_name {
            form.insert("CustomFriendlyName", name.as_str());
        }
        form.insert("To", number);
        form.insert("Channel", channel.as_str());
        if !self.custom_code {
            self.initiate_request(&form).await?;
            return Ok(())
        }
        let mut verification = Self::Verification::new(contact, None, String::new());
        let code = verification.as_str();
        form.insert("CustomCode", code.as_str());
        let res = self.initiate_request(&form).await?;
        verification.id = res.id;
        db.create_item(verification).await.map_err(Self::Error::err)?;
        Ok(())
    }

    async fn verify<DB: GetItem<Self::Verification>>(&self, contact: &Phone,  code: &str, db: &DB) -> Result<(), Self::Error> {
        if !self.custom_code {
            let number = contact.as_str();
            let mut form = HashMap::new();
            form.insert("To", number);
            form.insert("Code", code);
            return self.verify_request(&form, None).await
        }
        let contact = Contact::Phone(contact.clone());
        let key = Key::Pk(&contact);
        let verification = db.get_item(key).await.map_err(Self::Error::err)?;
        let saved_code = verification.as_str();
        if saved_code.as_str() != code {
            return Err(Error::InvalidCode)
        }
        let form = [("Status", "approved")].into();
        let id = Some(verification.id.as_str());
        self.verify_request(&form, id).await?;
        Ok(())
    }
}
