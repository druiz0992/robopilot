use notification_hub::models::hub::HubChannelName;

const DEFAULT_DELAY_MILLIS: u64 = 100;
const DEFAULT_PERIOD_MILLIS: u64 = 1000;
const DEFAULT_N_DIMS: usize = 3;

#[derive(Debug, Clone)]
pub struct ClientPipeOptions {
    write_pipe_path: Option<String>,
    read_pipe_path: Option<String>,
    channel: HubChannelName,
    n_dims: usize,
    delay_millis: u64,
    period_millis: u64,
}

impl ClientPipeOptions {
    pub fn new(
        write_path: Option<&str>,
        read_path: Option<&str>,
        channel: &str,
        n_dims: usize,
        delay_millis: u64,
        period_millis: u64,
    ) -> Result<Self, String> {
        let channel = HubChannelName::try_from(channel)?;
        Ok(Self {
            write_pipe_path: write_path.map(|s| s.to_string()),
            read_pipe_path: read_path.map(|s| s.to_string()),
            n_dims,
            channel,
            delay_millis,
            period_millis,
        })
    }
    pub fn write_path(&self) -> Option<String> {
        self.write_pipe_path.clone()
    }
    pub fn read_path(&self) -> Option<String> {
        self.read_pipe_path.clone()
    }
    pub fn delay(&self) -> u64 {
        self.delay_millis
    }
    pub fn period(&self) -> u64 {
        self.period_millis
    }
    pub fn channel(&self) -> HubChannelName {
        self.channel.clone()
    }
    pub fn n_dims(&self) -> usize {
        self.n_dims
    }
}

#[derive(Debug, Clone)]
pub struct ClientPipeOptionsBuilder {
    write_pipe_path: Option<String>,
    read_pipe_path: Option<String>,
    channel: Option<HubChannelName>,
    n_dims: Option<usize>,
    delay_millis: Option<u64>,
    period_millis: Option<u64>,
}

impl ClientPipeOptionsBuilder {
    pub fn new() -> Self {
        Self {
            write_pipe_path: None,
            read_pipe_path: None,
            channel: None,
            n_dims: None,
            delay_millis: None,
            period_millis: None,
        }
    }

    pub fn write_path(&self, path: &str) -> Self {
        let mut new = self.clone();
        new.write_pipe_path = Some(path.to_string());
        new
    }
    pub fn read_path(&self, path: &str) -> Self {
        let mut new = self.clone();
        new.read_pipe_path = Some(path.to_string());
        new
    }

    pub fn delay(&self, delay: u64) -> Self {
        let mut new = self.clone();
        new.delay_millis = Some(delay);
        new
    }
    pub fn period(&self, period: u64) -> Self {
        let mut new = self.clone();
        new.period_millis = Some(period);
        new
    }
    pub fn channel(&self, channel: &str) -> Result<Self, String> {
        let mut new = self.clone();
        new.channel = Some(
            HubChannelName::try_from(channel).map_err(|_| "Invalid channel name".to_string())?,
        );
        Ok(new)
    }
    pub fn n_dims(&self, n_dims: usize) -> Self {
        let mut new = self.clone();
        new.n_dims = Some(n_dims);
        new
    }
    pub fn build(self) -> Result<ClientPipeOptions, String> {
        if self.channel.is_none() {
            return Err("Invaluid channel name".to_string());
        }
        Ok(ClientPipeOptions {
            write_pipe_path: self.write_pipe_path,
            read_pipe_path: self.read_pipe_path,
            channel: self.channel.unwrap(),
            delay_millis: self.delay_millis.unwrap_or(DEFAULT_DELAY_MILLIS),
            period_millis: self.period_millis.unwrap_or(DEFAULT_PERIOD_MILLIS),
            n_dims: self.n_dims.unwrap_or(DEFAULT_N_DIMS),
        })
    }
}
