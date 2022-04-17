#include <Windows.h>
#include <fstream>

constexpr auto SERVICE_NAME = L"AutoPower";

SERVICE_STATUS serviceStatus = { 0 };
SERVICE_STATUS_HANDLE statusHandle = NULL;
HANDLE serviceStopEvent = INVALID_HANDLE_VALUE;

typedef DWORD(*PowerSetActiveSchemeFunc)(HKEY UserRootPowerKey /*ignore*/, const GUID* SchemeGuid);
PowerSetActiveSchemeFunc PowerSetActiveScheme;

inline void DebugString(const wchar_t* msg) {
	OutputDebugString(msg);
}

template<class... Format>
inline void FDebugString(wchar_t* msg, Format...args) {
	wsprintf(msg, args...);
	DebugString(msg);
}

inline void SetServiceStatusAndCheck(SERVICE_STATUS_HANDLE handle, SERVICE_STATUS* status) {
	if (SetServiceStatus(statusHandle, &serviceStatus) == FALSE) {
		FDebugString((wchar_t*)L"%s: ServiceMain: SetServiceStatus returned error.", SERVICE_NAME);
	}
}

template<typename T>
inline void SetIfNotNull(T& toSet, T value) noexcept {
	if (value) toSet = value;
}

inline void SetStatus(SERVICE_STATUS to, bool zero = false) {
	if (zero) ZeroMemory(&serviceStatus, sizeof(serviceStatus));
	SetIfNotNull(serviceStatus.dwServiceType, to.dwServiceType);
	SetIfNotNull(serviceStatus.dwCurrentState, to.dwCurrentState);
	SetIfNotNull(serviceStatus.dwControlsAccepted, to.dwControlsAccepted);
	SetIfNotNull(serviceStatus.dwWin32ExitCode, to.dwWin32ExitCode);
	SetIfNotNull(serviceStatus.dwServiceSpecificExitCode, to.dwServiceSpecificExitCode);
	SetIfNotNull(serviceStatus.dwCheckPoint, to.dwCheckPoint);
	SetIfNotNull(serviceStatus.dwWaitHint, to.dwWaitHint);
	SetServiceStatusAndCheck(statusHandle, &serviceStatus);
}

void OnWallPower() {
	PowerSetActiveScheme(NULL, &GUID_MIN_POWER_SAVINGS);
	DebugString(L"System is now on wall power.");
}

void OnBatteryPower() {
	PowerSetActiveScheme(NULL, &GUID_TYPICAL_POWER_SAVINGS);
	DebugString(L"System is now on battery power.");
}

void HandlePowerEvent(DWORD eventType, LPVOID eventData) {
	if (eventType != PBT_POWERSETTINGCHANGE)
		return;

	auto pbs = (POWERBROADCAST_SETTING*)eventData;

	if (pbs->PowerSetting != GUID_ACDC_POWER_SOURCE)
		return;

	DWORD newPower = *pbs->Data;
	switch (newPower)
	{
	case SYSTEM_POWER_CONDITION::PoAc: OnWallPower(); break;
	case SYSTEM_POWER_CONDITION::PoDc: OnBatteryPower(); break;
	default: DebugString(L"System power changed with ignored value."); break;
	}
}

void HandleStop() {
	if (serviceStatus.dwCurrentState != SERVICE_RUNNING)
		return;

	SetStatus({
		.dwCurrentState = SERVICE_STOP_PENDING,
		.dwControlsAccepted = 0,
		.dwWin32ExitCode = 0,
		.dwCheckPoint = 4
		});

	SetEvent(serviceStopEvent);
}

std::wstring GetServiceDirectory(int bufSize = 128) {
	auto pathBuf = new wchar_t[bufSize];
	GetModuleFileName(NULL, pathBuf, bufSize);
	auto path = std::wstring(pathBuf);
	const auto seperator = path.rfind(L'\\');
	path.erase(seperator + 1);
	return path;
}

DWORD WINAPI ServiceCtrlHandler(DWORD ctrl, DWORD eventType, LPVOID eventData, LPVOID _context) {
	switch (ctrl)
	{
	case SERVICE_CONTROL_POWEREVENT:
		HandlePowerEvent(eventType, eventData);
		break;

	case SERVICE_CONTROL_STOP: [[unlikely]]
		HandleStop();
		break;

	default:
		break;
	}
	return NO_ERROR;
}

VOID WINAPI ServiceMain(DWORD arc, LPSTR* argv) {
	DWORD Status = E_FAIL;

	statusHandle = RegisterServiceCtrlHandlerEx(SERVICE_NAME, ServiceCtrlHandler, NULL);
	if (statusHandle == NULL) {
		return;
	}

	SetStatus({
		.dwServiceType = SERVICE_WIN32_OWN_PROCESS,
		.dwCurrentState = SERVICE_START_PENDING,
		.dwControlsAccepted = 0,
		.dwCheckPoint = 0
		}, true);

	serviceStopEvent = CreateEvent(NULL, TRUE, FALSE, NULL);
	if (serviceStopEvent == NULL) {
		SetStatus({
			.dwCurrentState = SERVICE_STOPPED,
			.dwControlsAccepted = 0,
			.dwWin32ExitCode = GetLastError(),
			.dwCheckPoint = 1,
			});
		return;
	}

	SetStatus({
		.dwCurrentState = SERVICE_RUNNING,
		.dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_POWEREVENT,
		.dwCheckPoint = 0
		});

	if (RegisterPowerSettingNotification(statusHandle, &GUID_ACDC_POWER_SOURCE, 1) == NULL)
		throw;

	auto powerLib = LoadLibrary(L"PowrProf.dll");
	if (powerLib == NULL)
		throw;

	PowerSetActiveScheme = (PowerSetActiveSchemeFunc)GetProcAddress(powerLib, "PowerSetActiveScheme");

	WaitForSingleObject(serviceStopEvent, INFINITE);

	//Cleanup
	if (serviceStopEvent != NULL) CloseHandle(serviceStopEvent);

	SetStatus({
		.dwCurrentState = SERVICE_STOPPED,
		.dwControlsAccepted = 0,
		.dwWin32ExitCode = 0,
		.dwCheckPoint = 3,
		});
}

int wmain(int argc, TCHAR* arcv[]) {
	SERVICE_TABLE_ENTRY ServiceTable[] = {
		{(LPWSTR)SERVICE_NAME, (LPSERVICE_MAIN_FUNCTION)ServiceMain},
		{NULL, NULL}
	};

	if (StartServiceCtrlDispatcher(ServiceTable) == FALSE) {
		return GetLastError();
	}

	return 0;
}