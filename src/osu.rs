use rosu_v2::prelude::*;

const CLIENT_ID: u64 = 34709;
const CLIENT_SECRET: &str = "fWXi26RUYVs1Gf8qMhWHQEvjK4PJdEsdRgrhPrTu";

struct Manager {
    osu: Osu,
}

impl Manager {
    pub(crate) async fn new() -> Self {
        let osu = Osu::new(CLIENT_ID, CLIENT_SECRET).await.unwrap();
        Self { osu }
    }

    /// Finds the source (aka album) given artist, title, mapper, and difficulty
    pub(crate) async fn get_source(&self, artist: String, title: String, mapper: String, difficulty: String) -> String {
        // TODO: somehow use title, artist, and mapper info to query osu.ppy.sh to find the specific
        // beatmap, get the beatmap metadata/info, and get the "source" (aka album) from the beatmap
        // info.
        let search_result: BeatmapsetSearchResult = self.osu.beatmapset_search()
            .status(None)
            .query("artist={artist} creator={creator} {title}")
            .await
            .unwrap();
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::format;

    use super::*;

    #[ignore]
    #[test]
    fn test_osu() {
        let artist = "Down";
        let creator = "Down";
        let title = "Rihan Rider";
        let query = format!("{artist} - {title} ({creator})");

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let m = Manager::new().await;
            let search_result: BeatmapsetSearchResult = m.osu.beatmapset_search()
                .status(None)
                .query(query)
                .await
                .unwrap();
            let first_result = search_result.mapsets.first().unwrap();
            let source = first_result.clone().source;
            dbg!(first_result);
            println!("SOURCE: {:?}", source);
            assert_eq!(source, "크레이지레이싱 카트라이더");
        });
    }
}


