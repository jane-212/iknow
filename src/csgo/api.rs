use anyhow::{anyhow, Context, Result};
use chrono::{NaiveDate, NaiveTime};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct CsgoApi {
    client: Client,
    time: NaiveTime,
    time_format: String,
    teams: Vec<i32>,
}

impl CsgoApi {
    pub fn new() -> Result<CsgoApi> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br"),
        );
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static("PostmanRuntime/7.32.3"),
        );
        headers.insert(header::HOST, HeaderValue::from_static("localhost"));
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("build request client failed")?;
        let time = NaiveTime::from_hms_opt(0, 0, 0)
            .ok_or(anyhow!("error default time"))
            .context("set zero time failed")?;
        let time_format = "%Y-%m-%d+%H:%M:%S".to_string();
        let teams = vec![6667, 5995, 12396, 4608, 5378, 8840, 5752];

        Ok(Self {
            client,
            time,
            time_format,
            teams,
        })
    }

    pub async fn get_matches_by_date(&self, date: &NaiveDate) -> Result<Vec<Match>> {
        let url = format!(
            "https://gwapi.pwesports.cn/eventcenter/app/csgo/event/getMatchList?matchTime={}",
            date.and_time(self.time).format(&self.time_format)
        );

        let matches = self
            .client
            .get(url)
            .header(header::HOST, "gwapi.pwesports.cn")
            .send()
            .await
            .context("send request failed")?
            .json::<Response>()
            .await
            .context("get response failed")?
            .result
            .match_response
            .dto_list
            .into_iter()
            .filter(|item| self.teams.contains(&item.team1id) || self.teams.contains(&item.team2id))
            .map(|item| {
                Match::new(
                    (item.team1dto.name, item.team1dto.logo_white),
                    (item.team2dto.name, item.team2dto.logo_white),
                    (item.start_time / 1000, item.bo, item.csgo_event_dto.name),
                )
            })
            .collect::<Vec<Match>>();

        Ok(matches)
    }
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct Match {
    team1: Team,
    team2: Team,
    info: Info,
}

impl Match {
    fn new(team1: impl Into<Team>, team2: impl Into<Team>, info: impl Into<Info>) -> Match {
        Self {
            team1: team1.into(),
            team2: team2.into(),
            info: info.into(),
        }
    }
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct Info {
    start_time: i64,
    bo: String,
    name: String,
}

impl Info {
    fn new(start_time: i64, bo: impl Into<String>, name: impl Into<String>) -> Info {
        Self {
            start_time,
            bo: bo.into(),
            name: name.into(),
        }
    }
}

impl From<(i64, String, String)> for Info {
    fn from(value: (i64, String, String)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct Team {
    name: String,
    logo: String,
}

impl Team {
    fn new(name: impl Into<String>, logo: impl Into<String>) -> Team {
        Self {
            name: name.into(),
            logo: logo.into(),
        }
    }
}

impl From<(String, String)> for Team {
    fn from(value: (String, String)) -> Self {
        Self::new(value.0, value.1)
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Response {
    pub code: i32,
    pub message: String,
    pub result: MatchResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MatchResult {
    pub match_response: MatchResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MatchResponse {
    pub dto_list: Vec<DtoList>,
    pub item_count: i32,
    #[serde(rename = "csgoEventDTO")]
    pub csgo_event_dto: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct DtoList {
    pub id: i32,
    pub match_id: i32,
    pub nami_match_id: Option<i32>,
    pub event_id: i32,
    pub start_time: i64,
    #[serde(rename = "team1Id")]
    pub team1id: i32,
    #[serde(rename = "team1DTO")]
    pub team1dto: TeamDto,
    #[serde(rename = "team2Id")]
    pub team2id: i32,
    #[serde(rename = "team2DTO")]
    pub team2dto: TeamDto,
    pub score1: Option<i32>,
    pub score2: Option<i32>,
    pub bo: String,
    pub star: i32,
    pub winner_team_id: Option<i32>,
    pub match_type: i32,
    pub status: i32,
    #[serde(rename = "statsDTOList")]
    pub stats_dtolist: Option<String>,
    #[serde(rename = "csgoEventDTO")]
    pub csgo_event_dto: CsgoEventDto,
    #[serde(rename = "matchDetailDTO")]
    pub match_detail_dto: Option<String>,
    #[serde(rename = "mapBPDTOS")]
    pub map_bpdtos: Option<String>,
    #[serde(rename = "singleMatchDataDTOS")]
    pub single_match_data_dtos: Option<String>,
    pub subscribe_status: Option<String>,
    pub room_id: Option<String>,
    pub platform: Option<String>,
    pub stage_id: Option<String>,
    pub news_id: i32,
    pub match_id_list: Option<String>,
    pub completed_status: Option<String>,
    pub has_predict: bool,
    pub performance_stats_list: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct TeamDto {
    pub id: i32,
    pub team_id: i32,
    pub name: String,
    pub logo_black: String,
    pub logo_white: String,
    pub rank: Option<i32>,
    pub location: Option<String>,
    #[serde(rename = "playerDTOList")]
    pub player_dtolist: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CsgoEventDto {
    pub id: i32,
    pub event_id: i32,
    pub name: String,
    pub thumbnail: String,
    pub logo: String,
    pub start_time: i64,
    pub end_time: i64,
    pub prize: Option<String>,
    #[serde(rename = "regionDTO")]
    pub region_dto: Option<String>,
    pub team_number: Option<String>,
    pub publish_type: Option<i32>,
    pub scheduled_time: Option<String>,
    pub weight: Option<String>,
    pub publish_time: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "teamDTOList")]
    pub team_dtolist: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<i32>,
    pub topic: Option<String>,
    pub name_zh: Option<String>,
    pub event_type: Option<i32>,
    pub hot: bool,
    pub important: bool,
    pub background: Option<String>,
    pub event_sub_type: Option<i32>,
    pub live_type: Option<i32>,
    pub description: Option<String>,
    pub prize_list: Vec<Option<String>>,
}
