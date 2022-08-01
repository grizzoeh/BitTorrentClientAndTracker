const peersConnectedTime = document.getElementById(
  "peers-connected-time-chart"
);
const peersConnectedGroupedBy = document.getElementById(
  "peers-connected-groupedby-chart"
);
const peersCompletedTime = document.getElementById(
  "peers-completed-time-chart"
);
const torrentsTotalTime = document.getElementById("torrents-total-time-chart");
const torrentsTotalGroupedBy = document.getElementById(
  "torrents-total-groupedby-chart"
);
const torrentSelect = document.getElementById("torrent-select");

const ERROR = -1;
const ONE_DAY = 86400000;
const ONE_HOUR = 3600000;
const ONE_HOUR_HOURS = 1;
const FIVE_HOURS_HOURS = 5;
const ONE_DAY_HOURS = 24;
const THREE_DAYS_HOURS = 72;
const ONE_HOUR_MINUTES = 60;
const FIVE_HOURS_MINUTES = 300;
const ONE_DAY_MINUTES = 1440;
const THREE_DAYS_MINUTES = 4320;

// STATES

let connectedTime = "last_three_days";
let connectedGroupedBy = "hours";
let completedTime = "last_three_days";
let torrentTime = "last_three_days";
let torrentGroupedBy = "hours";
let torrentSelected = "";

// CHARTS REFERENCES

let connectedChart, completedChart, torrentsChart;

// LISTENERS CONNECTED

peersConnectedGroupedBy.addEventListener("change", (e) =>
  updateConnectedChart(e)
);
peersConnectedTime.addEventListener("change", (e) => updateConnectedChart(e));
torrentSelect.addEventListener("change", (e) => updateConnectedChart(e));

const updateConnectedChart = (e) => {
  connectedTime = e.target.id.includes("time") ? e.target.value : connectedTime;
  connectedGroupedBy = e.target.id.includes("groupedby")
    ? e.target.value
    : connectedGroupedBy;

  if (e.target.id === "torrent-select") {
    torrentSelected = e.target.value.split(",").map((t) => Number(t));
  }

  getStats()
    .then((data) => {
      const info_hashes = getAllInfoHashFilteredBy(
        data,
        toTimestamp(connectedTime)
      );
      if (info_hashes === ERROR) return;

      let html = "";
      let labels = [];
      info_hashes.map((torrent) => {
        if (infoHashesAreEqual(torrent, torrentSelected)) {
          html += `<option selected value="${torrent}">${torrent}</option>`;
          torrentSelected = torrent;
          labels.push(...getConnectedPeersFromTorrent(data, torrentSelected));
        } else {
          html += `<option value="${torrent}">${torrent}</option>`;
        }
      });

      torrentSelect.innerHTML = html;

      if (info_hashes.length === 0) {
        torrentSelect.innerHTML = `<option selected disabled value="">There are no Torrents with Data</option>`;
      }

      labels = groupedBy(labels, connectedGroupedBy, connectedTime);

      let data_aux = [];
      labels = labels.map((l) => {
        data_aux.push(l[0]);
        return l[1];
      });

      const datasets = [
        {
          label: "Connected Peers",
          data: data_aux,
          fill: false,
          borderColor: "rgb(75, 192, 192)",
          tension: 0.1,
        },
      ];

      connectedChart.data.labels = labels;
      connectedChart.data.datasets = datasets;
      connectedChart.update();
    })
    .catch((e) =>
      console.error("Error getting stats from connected listener: ", e)
    );
};

// LISTENERS COMPLETED

peersCompletedTime.addEventListener("change", (e) => updateCompletedChart(e));

const updateCompletedChart = (e) => {
  completedTime = e.target.id.includes("time") ? e.target.value : completedTime;

  getStats()
    .then((data) => {
      let labels = getAllInfoHashFilteredBy(data, toTimestamp(completedTime));
      let data_aux = [];
      labels = labels.map((torrent) => {
        data_aux.push(getTotalCompletedPeersFromTorrent(data, torrent));
        return [torrent[0], torrent[1], "...", torrent[labels.length - 1]];
      });

      const datasets = [
        {
          label: "Completed Peers",
          data: data_aux,
          backgroundColor: [
            "rgba(255, 99, 132, 0.2)",
            "rgba(54, 162, 235, 0.2)",
            "rgba(255, 206, 86, 0.2)",
            "rgba(75, 192, 192, 0.2)",
            "rgba(153, 102, 255, 0.2)",
            "rgba(255, 159, 64, 0.2)",
          ],
          borderColor: [
            "rgba(255, 99, 132, 1)",
            "rgba(54, 162, 235, 1)",
            "rgba(255, 206, 86, 1)",
            "rgba(75, 192, 192, 1)",
            "rgba(153, 102, 255, 1)",
            "rgba(255, 159, 64, 1)",
          ],
          borderWidth: 1,
        },
      ];

      completedChart.data.labels = labels;
      completedChart.data.datasets = datasets;
      completedChart.update();
    })
    .catch((e) =>
      console.error("Error getting stats from completed listener: ", e)
    );
};

// LISTENERS TORRENT

torrentsTotalGroupedBy.addEventListener("change", (e) => updateTorrentChart(e));
torrentsTotalTime.addEventListener("change", (e) => updateTorrentChart(e));

const updateTorrentChart = (e) => {
  torrentTime = e.target.id.includes("time") ? e.target.value : torrentTime;
  torrentGroupedBy = e.target.id.includes("time")
    ? torrentGroupedBy
    : e.target.value;

  getStats()
    .then((data) => {
      const timestamps = getAllTorrents(data);
      const filteredTimestamps = filterByDate(timestamps, torrentTime);
      if (filteredTimestamps === ERROR) return;

      labels = groupedBy(filteredTimestamps, torrentGroupedBy, torrentTime);

      let data_aux = [];
      labels = labels.map((l) => {
        data_aux.push(l[0]);
        return l[1];
      });

      const datasets = [
        {
          label: "Total of Torrents",
          data: data_aux,
          fill: false,
          borderColor: "#fc9403",
          tension: 0.1,
        },
      ];

      torrentsChart.data.labels = labels;
      torrentsChart.data.datasets = datasets;
      torrentsChart.update();
    })
    .catch((e) =>
      console.error("Error getting stats from torrents listener: ", e)
    );
};

// DATA PROCESSING

const filterByDate = (timestamps, date) => {
  let limit = toTimestamp(date);
  if (limit === ERROR) return ERROR;

  let result = [];
  const now = Date.now();
  timestamps.map((t) => {
    if (now - t * 1000 <= limit) {
      result.push(t);
    }
  });

  return result;
};

const getStats = async () => {
  const data = await fetch("http://localhost:8088/stats/data");
  const dataJson = await data.json();
  return dataJson;
};

const getAllTorrents = (data) => {
  return data["historical_torrents"];
};

const getAllPeers = (data) => {
  let result = [];
  data["historical_peers"].map((peer) => {
    result.push(...peer[1][0][1]);
  });
  return result;
};

const getAllInfoHashFilteredBy = (data, filterBy) => {
  let result = [];
  const now = Date.now();
  data["historical_peers"].map((torrent) => {
    torrent[1][0][1].map((peer) => {
      if (
        result[result.length - 1] !== torrent[0] &&
        now - peer * 1000 <= filterBy
      ) {
        result.push(torrent[0]);
      }
    });
  });
  return result;
};

const infoHashesAreEqual = (l1, l2) => {
  for (let i = 0; i < l1.length; i++) {
    if (l1[i] !== l2[i]) return false;
  }
  return true;
};

const getConnectedPeersFromTorrent = (data, torrent) => {
  let result = [];
  data["historical_peers"].map((peer) => {
    if (infoHashesAreEqual(peer[0], torrent)) {
      peer[1].map((peer) => {
        if (peer[0] === "connected") result.push(...peer[1]);
      });
      return result;
    }
  });
  return result;
};

const getTotalCompletedPeersFromTorrent = (data, torrent) => {
  let counter = 0;
  data["historical_peers"].map((peer) => {
    if (peer[0] === torrent) {
      peer[1].map((peer2) => {
        if (peer2[0] === "completed") counter += peer2[1].length;
      });
      return counter;
    }
  });
  return counter;
};

// CHARTS

const createBarChart = (id, data) => {
  const ctx = document.getElementById(id).getContext("2d");
  const config = {
    type: "bar",
    data,
    options: {
      scales: {
        y: {
          beginAtZero: true,
        },
      },
    },
  };

  return new Chart(ctx, config);
};

const createLineChart = (id, data) => {
  const ctx = document.getElementById(id).getContext("2d");
  const config = {
    type: "line",
    data: data,
  };
  return new Chart(ctx, config);
};

const toTimestamp = (date) => {
  switch (date) {
    case "last_hour":
      return ONE_HOUR;
    case "last_five_hours":
      return 5 * ONE_HOUR;
    case "last_day":
      return ONE_DAY;
    case "last_three_days":
      return 3 * ONE_DAY;
    default:
      console.error("Filter by date did not match: ", date);
      return ERROR;
  }
};

const prepareDataForPeersConnectedLineChart = (data) => {
  const info_hashes = getAllInfoHashFilteredBy(
    data,
    toTimestamp(connectedTime)
  );

  let html = "";
  let labels = [];
  info_hashes.map((torrent, index) => {
    if (index === 0) {
      html += `<option selected value="${torrent}">${torrent}</option>`;
      torrentSelected = torrent;
      labels.push(...getConnectedPeersFromTorrent(data, torrent));
    } else {
      html += `<option value="${torrent}">${torrent}</option>`;
    }
  });

  torrentSelect.innerHTML = html;

  if (info_hashes.length === 0) {
    torrentSelect.innerHTML = `<option selected disabled value="">There are no Torrents</option>`;
  }

  labels = groupedBy(labels, connectedGroupedBy, connectedTime);

  let data_aux = [];
  labels = labels.map((l) => {
    data_aux.push(l[0]);
    return l[1];
  });

  const datasets = [
    {
      label: "Connected Peers",
      data: data_aux,
      fill: false,
      borderColor: "rgb(75, 192, 192)",
      tension: 0.1,
    },
  ];

  const result = {
    labels,
    datasets,
  };

  return result;
};

const getHours = (time) => {
  switch (time) {
    case "last_hour":
      return ONE_HOUR_HOURS;
    case "last_day":
      return ONE_DAY_HOURS;
    case "last_five_hours":
      return FIVE_HOURS_HOURS;
    case "last_three_days":
      return THREE_DAYS_HOURS;
    default:
      console.error("Get hours did not match: ", time);
      break;
  }
};

const getMinutes = (time) => {
  switch (time) {
    case "last_hour":
      return ONE_HOUR_MINUTES;
    case "last_day":
      return ONE_DAY_MINUTES;
    case "last_five_hours":
      return FIVE_HOURS_MINUTES;
    case "last_three_days":
      return THREE_DAYS_MINUTES;
    default:
      console.error("Get minutes did not match: ", time);
      break;
  }
};

const formatHour = (day, hour, minute) => {
  if (day.length === 1) day = "0" + day;
  if (hour.length === 1) hour = "0" + hour;
  if (minute.length === 1) minute = "0" + minute;
  return `${day === "" ? day : day + " /"} ${hour}:${minute}`;
};

const getCorrectDay = (day) => {
  if (day === 0) return 31;
  if (day === -1) return 30;
  if (day === -2) return 29;
  return day;
};

const formatHourTime = (time) => {
  if (time === -5) return 19;
  if (time === -4) return 20;
  if (time === -3) return 21;
  if (time === -2) return 22;
  if (time === -1) return 23;
  return time;
};

const groupedBy = (data, groupedBy, time) => {
  let data_copy = [...data];
  data_copy.sort();
  let result = [];
  let counter = 0;
  let subFiveHours = time === "last_five_hours" ? 5 : 0;
  const currentDay = new Date().getDate();
  const currentHour = new Date().getHours();
  const currentMinute = new Date().getMinutes();

  if (groupedBy === "hours") {
    if (time === "last_three_days") {
      for (let j = 2; j >= 0; j--) {
        let daysCompleted = 0;
        for (let i = currentHour; i < ONE_DAY_HOURS + currentHour + 1; i++) {
          data_copy.map((value) => {
            const valueDay = new Date(value * 1000).getDate();
            const valueHour = new Date(value * 1000).getHours();
            if (
              getCorrectDay(currentDay - j) === valueDay &&
              valueHour === i - 1
            ) {
              counter++;
            }
          });
          if (i % 24 === 0) {
            daysCompleted++;
          }
          result.push([
            counter,
            formatHour(
              getCorrectDay(currentDay - j + daysCompleted),
              `${formatHourTime(((i - 1) % 24) - subFiveHours)}`,
              "00"
            ),
          ]);
        }
      }
    } else {
      for (let i = currentHour; i < getHours(time) + currentHour + 1; i++) {
        data_copy.map((value) => {
          const valueDay = new Date(value * 1000).getDate();
          const valueHour = new Date(value * 1000).getHours();
          if (
            (valueDay === currentDay || valueDay === currentDay - 1) &&
            valueHour === i - 1
          ) {
            counter++;
          }
        });
        result.push([
          counter,
          formatHour("", `${formatHourTime((i % 24) - subFiveHours)}`, "00"),
        ]);
      }
    }
  } else {
    if (time === "last_three_days") {
      for (let k = 2; k >= 0; k--) {
        for (let j = currentHour; j < ONE_DAY_HOURS + currentHour + 1; j++) {
          let daysCompleted = 0;
          for (let i = currentMinute; i < 60 + currentMinute; i++) {
            data_copy.map((value) => {
              const valueDay = new Date(value * 1000).getDate();
              const valueHour = new Date(value * 1000).getHours();
              const valueMinute = new Date(value * 1000).getMinutes();
              if (
                getCorrectDay(currentDay - k) === valueDay &&
                valueHour === j % 24 &&
                valueMinute === i % 60
              ) {
                counter++;
              }
            });
            if (j % 24 === 0) {
              daysCompleted++;
            }
            result.push([
              counter,
              formatHour(
                getCorrectDay(currentDay - k + daysCompleted),
                `${formatHourTime((j % 24) - subFiveHours)}`,
                `${i % 60}`
              ),
            ]);
          }
        }
      }
    } else if (time === "last_day") {
      for (let j = currentHour; j < 24 + currentHour + 1; j++) {
        for (let i = currentMinute; i < 60 + currentMinute; i++) {
          data_copy.map((value) => {
            const valueDay = new Date(value * 1000).getDate();
            const valueHour = new Date(value * 1000).getHours();
            const valueMinute = new Date(value * 1000).getMinutes();
            if (
              (valueDay === currentDay || valueDay === currentDay - 1) &&
              valueHour === j % 24 &&
              valueMinute === i % 60
            ) {
              counter++;
            }
          });
          result.push([
            counter,
            formatHour(
              "",
              `${formatHourTime((j % 24) - subFiveHours)}`,
              `${i % 60}`
            ),
          ]);
        }
      }
    } else if (time === "last_five_hours") {
      for (let j = currentHour; j < 6 + currentHour; j++) {
        for (let i = currentMinute; i < 60 + currentMinute; i++) {
          data_copy.map((value) => {
            const valueDay = new Date(value * 1000).getDate();
            const valueHour = new Date(value * 1000).getHours();
            const valueMinute = new Date(value * 1000).getMinutes();
            if (
              (valueDay === currentDay || valueDay === currentDay - 1) &&
              valueHour === j % 24 &&
              valueMinute === i % 60
            ) {
              counter++;
            }
          });
          result.push([
            counter,
            formatHour(
              "",
              `${formatHourTime((j % 24) - subFiveHours)}`,
              `${i % 60}`
            ),
          ]);
        }
      }
    } else {
      for (let j = currentHour; j < 2 + currentHour; j++) {
        for (let i = currentMinute; i < 60 + currentMinute; i++) {
          data_copy.map((value) => {
            const valueDay = new Date(value * 1000).getDate();
            const valueHour = new Date(value * 1000).getHours();
            const valueMinute = new Date(value * 1000).getMinutes();
            if (
              (valueDay === currentDay || valueDay === currentDay - 1) &&
              valueHour === j % 24 &&
              valueMinute === i % 60
            ) {
              counter++;
            }
          });
          result.push([
            counter,
            formatHour(
              "",
              `${formatHourTime((j % 24) - subFiveHours - 1)}`,
              `${i % 60}`
            ),
          ]);
        }
      }
    }
  }
  return result;
};

const prepareDataForFullDownloadBarChar = (data) => {
  let labels = getAllInfoHashFilteredBy(data, toTimestamp(completedTime));
  let data_aux = [];
  labels = labels.map((torrent) => {
    data_aux.push(getTotalCompletedPeersFromTorrent(data, torrent));
    return [torrent[0], torrent[1], "...", torrent[labels.length - 1]];
  });

  const result = {
    labels,
    datasets: [
      {
        label: "Completed Peers",
        data: data_aux,
        backgroundColor: [
          "rgba(255, 99, 132, 0.2)",
          "rgba(54, 162, 235, 0.2)",
          "rgba(255, 206, 86, 0.2)",
          "rgba(75, 192, 192, 0.2)",
          "rgba(153, 102, 255, 0.2)",
          "rgba(255, 159, 64, 0.2)",
        ],
        borderColor: [
          "rgba(255, 99, 132, 1)",
          "rgba(54, 162, 235, 1)",
          "rgba(255, 206, 86, 1)",
          "rgba(75, 192, 192, 1)",
          "rgba(153, 102, 255, 1)",
          "rgba(255, 159, 64, 1)",
        ],
        borderWidth: 1,
      },
    ],
  };

  return result;
};

const prepareDataForTotalOfTorrentsLineChar = (data) => {
  const torrents = getAllTorrents(data);
  const torrents_aux = filterByDate(torrents, torrentTime);
  if (torrents_aux === ERROR) return console.error("Option did not match");

  labels = groupedBy(torrents_aux, connectedGroupedBy, torrentTime);

  let data_aux = [];
  labels = labels.map((l) => {
    data_aux.push(l[0]);
    return l[1];
  });

  const datasets = [
    {
      label: "Total of Torrents",
      data: data_aux,
      fill: false,
      borderColor: "#fc9403",
      tension: 0.1,
    },
  ];

  const result = {
    labels,
    datasets,
  };

  return result;
};

// INITIALIZE CHARTS

const getCharts = () => {
  getStats()
    .then((data) => {
      connectedChart = createLineChart(
        "peers-connected-chart",
        prepareDataForPeersConnectedLineChart(data)
      );
      completedChart = createBarChart(
        "peers-full-download-chart",
        prepareDataForFullDownloadBarChar(data)
      );
      torrentsChart = createLineChart(
        "total-of-torrents-chart",
        prepareDataForTotalOfTorrentsLineChar(data)
      );
    })
    .catch((e) => console.error("Error getting stats: ", e));
};

getCharts();
