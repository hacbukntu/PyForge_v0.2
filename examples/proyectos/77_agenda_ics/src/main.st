// Exportar evento de calendario
// Genera un evento iCalendar de ejemplo con horas UTC explícitas.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
// Fechas UTC de ejemplo; no se calculan zonas horarias ni recurrencias.
variable inicio = "20300115T140000Z";
variable fin = "20300115T150000Z";
variable evento = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ST//Ejemplos//ES\r\nBEGIN:VEVENT\r\nUID:reunion-001@example.test\r\nDTSTAMP:20300101T000000Z\r\nDTSTART:" + inicio + "\r\nDTEND:" + fin + "\r\nSUMMARY:Revision ST\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
io.escribir_texto("evento.ics",evento);
mostrar("Calendario generado");
