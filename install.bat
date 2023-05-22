schtasks.exe /Create /TN "Hodiny" /ST 00:00 /SC minute /MO 15 /TR "%~dp0\hodiny.exe"
