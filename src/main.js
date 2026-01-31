const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let resetTimes = { fiveHour: null, sevenDay: null };
let elements = {};
let profileModalOpen = false;
const COMPACT_HEIGHT = 109;
const PROFILE_HEADER_HEIGHT = 45;
const PROFILE_ITEM_HEIGHT = 40;

function getColorClass(percent) {
  if (percent >= 80) return 'red';
  if (percent >= 50) return 'yellow';
  return 'green';
}

function formatTime(resetAt) {
  if (!resetAt) return '--';
  const now = new Date();
  const reset = new Date(resetAt);
  const diffMs = reset - now;
  if (diffMs <= 0) return '0m';

  const diffSec = Math.floor(diffMs / 1000);
  const days = Math.floor(diffSec / 86400);
  const hours = Math.floor((diffSec % 86400) / 3600);
  const minutes = Math.floor((diffSec % 3600) / 60);

  if (days > 0) return `${days}d${hours}h`;
  if (hours > 0) return `${hours}h${minutes}m`;
  return `${minutes}m`;
}

function getTimerPercent(resetAt, totalHours) {
  if (!resetAt) return 0;
  const now = new Date();
  const reset = new Date(resetAt);
  const diffMs = reset - now;
  if (diffMs <= 0) return 0;
  const totalMs = totalHours * 60 * 60 * 1000;
  return Math.min(100, (diffMs / totalMs) * 100);
}

function updateUsage(usage) {
  elements.loading.classList.add('hidden');
  elements.usageContainer.classList.remove('hidden');

  if (usage.five_hour) {
    const percent = Math.round(usage.five_hour.utilization || 0);
    const color = getColorClass(percent);
    elements.fiveHourBar.style.width = `${percent}%`;
    elements.fiveHourBar.className = `bar-fill ${color}`;
    elements.fiveHourPercent.textContent = `${percent}%`;
    elements.fiveHourPercent.className = `value ${color}`;
    resetTimes.fiveHour = usage.five_hour.resets_at;
  }

  if (usage.seven_day) {
    const percent = Math.round(usage.seven_day.utilization || 0);
    const color = getColorClass(percent);
    elements.sevenDayBar.style.width = `${percent}%`;
    elements.sevenDayBar.className = `bar-fill ${color}`;
    elements.sevenDayPercent.textContent = `${percent}%`;
    elements.sevenDayPercent.className = `value ${color}`;
    resetTimes.sevenDay = usage.seven_day.resets_at;
  }

  updateTimers();
}

function updateTimers() {
  if (resetTimes.fiveHour) {
    const timerPercent = getTimerPercent(resetTimes.fiveHour, 5);
    elements.fiveHourTimerBar.style.width = `${timerPercent}%`;
    elements.fiveHourTimer.textContent = formatTime(resetTimes.fiveHour);
  }

  if (resetTimes.sevenDay) {
    const timerPercent = getTimerPercent(resetTimes.sevenDay, 168);
    elements.sevenDayTimerBar.style.width = `${timerPercent}%`;
    elements.sevenDayTimer.textContent = formatTime(resetTimes.sevenDay);
  }
}

function showError(msg) {
  elements.loading.classList.add('hidden');
  elements.errorContainer.classList.remove('hidden');
  elements.errorText.textContent = msg;
}

async function showProfileModal() {
  const profiles = await invoke('get_chrome_profiles');
  const selectedProfile = await invoke('get_selected_profile') || 'Default';

  const profileList = document.getElementById('profile-list');
  profileList.innerHTML = '';

  profiles.forEach(profile => {
    const div = document.createElement('div');
    div.className = 'profile-item' + (profile.id === selectedProfile ? ' selected' : '');
    div.innerHTML = `
      <div class="profile-name">${profile.name}</div>
      ${profile.email ? `<div class="profile-email">${profile.email}</div>` : ''}
    `;
    div.addEventListener('click', async () => {
      await invoke('set_selected_profile', { profileId: profile.id });
      hideProfileModal();
    });
    profileList.appendChild(div);
  });

  // Calculate height based on number of profiles
  const expandedHeight = PROFILE_HEADER_HEIGHT + (profiles.length * PROFILE_ITEM_HEIGHT);
  await invoke('set_window_height', { height: expandedHeight });

  // Switch views - hide main, show profiles
  document.getElementById('main-view').classList.add('hidden');
  document.getElementById('profile-modal').classList.remove('hidden');
  profileModalOpen = true;
}

async function hideProfileModal() {
  // Switch views back - hide profiles, show main
  document.getElementById('profile-modal').classList.add('hidden');
  document.getElementById('main-view').classList.remove('hidden');
  profileModalOpen = false;

  // Shrink window back to compact size
  await invoke('set_window_height', { height: COMPACT_HEIGHT });
}

async function checkFirstRun() {
  const selectedProfile = await invoke('get_selected_profile');
  if (!selectedProfile) {
    showProfileModal();
  }
}

window.addEventListener('DOMContentLoaded', async () => {
  elements = {
    loading: document.getElementById('loading'),
    usageContainer: document.getElementById('usage-container'),
    errorContainer: document.getElementById('error-container'),
    errorText: document.getElementById('error-text'),
    fiveHourBar: document.getElementById('five-hour-bar'),
    fiveHourPercent: document.getElementById('five-hour-percent'),
    fiveHourTimerBar: document.getElementById('five-hour-timer-bar'),
    fiveHourTimer: document.getElementById('five-hour-timer'),
    sevenDayBar: document.getElementById('seven-day-bar'),
    sevenDayPercent: document.getElementById('seven-day-percent'),
    sevenDayTimerBar: document.getElementById('seven-day-timer-bar'),
    sevenDayTimer: document.getElementById('seven-day-timer'),
  };

  // Update timers immediately when window becomes visible
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) {
      updateTimers();
    }
  });

  // Also update when window gains focus
  window.addEventListener('focus', updateTimers);

  // Button handlers
  document.getElementById('btn-profile').addEventListener('click', () => {
    if (profileModalOpen) {
      hideProfileModal();
    } else {
      showProfileModal();
    }
  });
  document.getElementById('btn-home').addEventListener('click', () => {
    invoke('open_url', { url: 'https://claude.ai/' });
  });
  document.getElementById('btn-settings').addEventListener('click', () => {
    invoke('open_url', { url: 'https://claude.ai/settings/usage' });
  });

  await listen('usage-updated', (e) => {
    elements.errorContainer.classList.add('hidden');
    updateUsage(e.payload);
  });

  await listen('usage-error', (e) => showError(e.payload));

  try {
    const usage = await invoke('get_usage');
    if (usage) updateUsage(usage);
    const error = await invoke('get_last_error');
    if (error) showError(error);
  } catch (e) {}

  setTimeout(async () => {
    if (!elements.loading.classList.contains('hidden')) {
      try { await invoke('refresh_usage'); } catch (e) { showError(String(e)); }
    }
  }, 2000);

  // Check if first run - show profile selector
  checkFirstRun();

  setInterval(updateTimers, 1000);
});
