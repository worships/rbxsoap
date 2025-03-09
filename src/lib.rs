use serde::{Deserialize, Serialize};
use serde_json;
use quick_xml::de::from_str;
use reqwest;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct RccConfig {
    pub base_url: String,
    pub soap_namespace: String,
}

impl Default for RccConfig {
    fn default() -> Self {
        Self {
            base_url: "roblox.com".to_string(),
            soap_namespace: "http://roblox.com/".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct RccSoapMessages {
    config: RccConfig,
    get_all_jobs_msg_template: &'static str,
    open_job_msg_template: &'static str,
    batch_job_msg_template: &'static str,
    close_job_msg_template: &'static str,
    execute_script_msg_template: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameOpenSettings {
    pub place_id: i64,
    pub creator_id: i64,
    pub game_id: String,
    pub machine_address: String,
    pub max_players: i32,
    pub gsm_interval: i32,
    pub max_game_instances: i32,
    pub preferred_player_capacity: i32,
    pub universe_id: i64,
    pub base_url: String,
    pub matchmaking_context_id: i32,
    pub creator_type: String,
    pub place_version: i32,
    pub job_id: String,
    pub preferred_port: i32,
    pub api_key: String,
    pub place_visit_access_key: String,
    pub place_fetch_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameOpenJson {
    pub mode: String,
    pub settings: GameOpenSettings,
}

impl RccSoapMessages {
    pub fn new() -> Self {
        Self::with_config(RccConfig::default())
    }

    pub fn with_config(config: RccConfig) -> Self {
        Self {
            config,
            get_all_jobs_msg_template: r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:rob="{namespace}">
   <soapenv:Header/>
   <soapenv:Body>
      <rob:GetAllJobs/>
   </soapenv:Body>
</soapenv:Envelope>"#,
            open_job_msg_template: r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:rob="{namespace}">
   <soapenv:Header/>
   <soapenv:Body>
      <rob:OpenJob>
         <rob:job>
            <rob:id>{job_id}</rob:id>
            <rob:expirationInSeconds>{job_expiration}</rob:expirationInSeconds>
            <rob:cores>{job_cores}</rob:cores>
         </rob:job>
         <rob:script>
            <rob:name>{script_name}</rob:name>
            <rob:script><![CDATA[
{run_script}
            ]]></rob:script>
            <rob:arguments>
                {arguments}
            </rob:arguments>
         </rob:script>
      </rob:OpenJob>
   </soapenv:Body>
</soapenv:Envelope>"#,
            batch_job_msg_template: r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:rob="{namespace}">
   <soapenv:Header/>
   <soapenv:Body>
      <rob:BatchJob>
         <rob:job>
            <rob:id>{job_id}</rob:id>
            <rob:expirationInSeconds>{job_expiration}</rob:expirationInSeconds>
            <rob:cores>{job_cores}</rob:cores>
         </rob:job>
         <rob:script>
            <rob:name>{script_name}</rob:name>
            <rob:script><![CDATA[
{run_script}
            ]]></rob:script>
            <rob:arguments>
                {arguments}
            </rob:arguments>
         </rob:script>
      </rob:BatchJob>
   </soapenv:Body>
</soapenv:Envelope>"#,
            close_job_msg_template: r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:rob="{namespace}">
   <soapenv:Header/>
   <soapenv:Body>
      <rob:CloseJob>
         <rob:jobID>{job_id}</rob:jobID>
      </rob:CloseJob>
   </soapenv:Body>
</soapenv:Envelope>"#,
            execute_script_msg_template: r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:rob="{namespace}">
   <soapenv:Header/>
   <soapenv:Body>
      <rob:Execute>
         <rob:jobID>{job_id}</rob:jobID>
         <rob:script>
            <rob:name>{script_name}</rob:name>
            <rob:script>{script}</rob:script>
            <rob:arguments>
               {arguments}
            </rob:arguments>
         </rob:script>
      </rob:Execute>
   </soapenv:Body>
</soapenv:Envelope>"#,
        }
    }

    fn format_template(&self, template: &str) -> String {
        template.replace("{namespace}", &self.config.soap_namespace)
    }

    pub fn generate_arguments(&self, arguments: &[LuaValue]) -> String {
        arguments
            .iter()
            .map(|arg| {
                let (arg_type, arg_value) = match arg {
                    LuaValue::Boolean(b) => ("LUA_TBOOLEAN", b.to_string()),
                    LuaValue::Number(n) => ("LUA_TNUMBER", n.to_string()),
                    LuaValue::String(s) => ("LUA_TSTRING", s.clone()),
                };
                format!(
                    r#"<rob:LuaValue>
                    <rob:type>{}</rob:type>
                    <rob:value>{}</rob:value>
                </rob:LuaValue>"#,
                    arg_type, arg_value
                )
            })
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn format_open_job_message(
        &self,
        job_id: &str,
        expiration: i32,
        cores: i32,
        script_name: &str,
        run_script: &str,
        arguments: &[LuaValue],
    ) -> String {
        let parsed_arguments = self.generate_arguments(arguments);
        let template = self.format_template(self.open_job_msg_template);
        template
            .replace("{job_id}", job_id)
            .replace("{job_expiration}", &expiration.to_string())
            .replace("{job_cores}", &cores.to_string())
            .replace("{script_name}", script_name)
            .replace("{run_script}", run_script)
            .replace("{arguments}", &parsed_arguments)
    }

    pub fn format_batch_job_message(
        &self,
        job_id: &str,
        expiration: i32,
        cores: i32,
        script_name: &str,
        run_script: &str,
        arguments: &[LuaValue],
    ) -> String {
        let parsed_arguments = self.generate_arguments(arguments);
        let template = self.format_template(self.batch_job_msg_template);
        template
            .replace("{job_id}", job_id)
            .replace("{job_expiration}", &expiration.to_string())
            .replace("{job_cores}", &cores.to_string())
            .replace("{script_name}", script_name)
            .replace("{run_script}", run_script)
            .replace("{arguments}", &parsed_arguments)
    }

    pub fn format_close_job_message(&self, job_id: &str) -> String {
        let template = self.format_template(self.close_job_msg_template);
        template.replace("{job_id}", job_id)
    }

    pub fn format_execute_script_message(
        &self,
        job_id: &str,
        script_name: &str,
        script: &str,
        arguments: &[LuaValue],
    ) -> String {
        let parsed_arguments = self.generate_arguments(arguments);
        let template = self.format_template(self.execute_script_msg_template);
        template
            .replace("{job_id}", job_id)
            .replace("{script_name}", script_name)
            .replace("{script}", script)
            .replace("{arguments}", &parsed_arguments)
    }

    pub fn format_game_open_json(
        &self,
        place_id: i64,
        creator_id: i64,
        job_id: &str,
        api_key: &str,
        max_players: i32,
        gsm_interval: i32,
        port_number: i32,
        creator_type: &str,
        place_version: i32,
        machine_address: &str,
        universe_id: Option<i64>,
    ) -> String {
        let universe_id = universe_id.unwrap_or(place_id);
        let settings = GameOpenSettings {
            place_id,
            creator_id,
            game_id: job_id.to_string(),
            machine_address: machine_address.to_string(),
            max_players,
            gsm_interval,
            max_game_instances: 1,
            preferred_player_capacity: max_players,
            universe_id,
            base_url: self.config.base_url.clone(),
            matchmaking_context_id: 1,
            creator_type: creator_type.to_string(),
            place_version,
            job_id: job_id.to_string(),
            preferred_port: port_number,
            api_key: api_key.to_string(),
            place_visit_access_key: "None".to_string(),
            place_fetch_url: format!("https://www.{}/asset/?id={}", self.config.base_url, place_id),
        };

        let game_open = GameOpenJson {
            mode: "GameServer".to_string(),
            settings,
        };

        serde_json::to_string(&game_open).unwrap()
    }

    pub fn parse_get_all_jobs_response(&self, response_text: &str) -> Vec<Job> {
        #[derive(Debug, Deserialize)]
        struct SoapEnvelope {
            #[serde(rename = "SOAP-ENV:Body")]
            body: SoapBody,
        }

        #[derive(Debug, Deserialize)]
        struct SoapBody {
            #[serde(rename = "ns1:GetAllJobsResponse")]
            response: GetAllJobsResponse,
        }

        #[derive(Debug, Deserialize)]
        struct GetAllJobsResponse {
            #[serde(rename = "ns1:GetAllJobsResult")]
            result: Option<JobsResult>,
        }

        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum JobsResult {
            Single(Job),
            Multiple(Vec<Job>),
        }

        if let Ok(envelope) = from_str::<SoapEnvelope>(response_text) {
            if let Some(result) = envelope.body.response.result {
                match result {
                    JobsResult::Single(job) => vec![job],
                    JobsResult::Multiple(jobs) => jobs,
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    pub fn format_get_all_jobs_message(&self) -> String {
        self.format_template(self.get_all_jobs_msg_template)
    }

    fn send_soap_request(&self, endpoint: &str, soap_message: &str) -> Result<String, Box<dyn Error>> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(endpoint)
            .header("Content-Type", "text/xml")
            .body(soap_message.to_string())
            .send()?
            .text()?;
        Ok(response)
    }

    pub fn get_all_jobs(&self, endpoint: &str) -> Result<Vec<Job>, Box<dyn Error>> {
        let message = self.format_get_all_jobs_message();
        let response = self.send_soap_request(endpoint, &message)?;
        Ok(self.parse_get_all_jobs_response(&response))
    }

    pub fn open_job(
        &self,
        endpoint: &str,
        job_id: &str,
        expiration: i32,
        cores: i32,
        script_name: &str,
        run_script: &str,
        arguments: &[LuaValue],
    ) -> Result<String, Box<dyn Error>> {
        let message = self.format_open_job_message(job_id, expiration, cores, script_name, run_script, arguments);
        self.send_soap_request(endpoint, &message)
    }

    pub fn batch_job(
        &self,
        endpoint: &str,
        job_id: &str,
        expiration: i32,
        cores: i32,
        script_name: &str,
        run_script: &str,
        arguments: &[LuaValue],
    ) -> Result<String, Box<dyn Error>> {
        let message = self.format_batch_job_message(job_id, expiration, cores, script_name, run_script, arguments);
        self.send_soap_request(endpoint, &message)
    }

    pub fn close_job(&self, endpoint: &str, job_id: &str) -> Result<String, Box<dyn Error>> {
        let message = self.format_close_job_message(job_id);
        self.send_soap_request(endpoint, &message)
    }

    pub fn execute_script(
        &self,
        endpoint: &str,
        job_id: &str,
        script_name: &str,
        script: &str,
        arguments: &[LuaValue],
    ) -> Result<String, Box<dyn Error>> {
        let message = self.format_execute_script_message(job_id, script_name, script, arguments);
        self.send_soap_request(endpoint, &message)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    #[serde(rename = "ns1:id")]
    pub id: String,
    #[serde(rename = "ns1:expirationInSeconds")]
    pub expiration_in_seconds: i32,
    #[serde(rename = "ns1:category")]
    pub category: String,
    #[serde(rename = "ns1:cores")]
    pub cores: i32,
}

#[derive(Debug, Clone)]
pub enum LuaValue {
    Boolean(bool),
    Number(f64),
    String(String),
}
