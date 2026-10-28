importar "math";
importar "io";
// Cálculo ilustrativo: intereses constantes, sin impuestos ni comisiones.
funcion saldo_futuro(capital: flotante, aporte: flotante, tasa: flotante, meses: entero) -> flotante {
    variable saldo = capital;
    para mes en rango(0, meses) { saldo = saldo * (1.0 + tasa) + aporte; }
    retornar saldo;
}
variable capital = flotante(io.leer_linea("Capital inicial: "));
variable aporte = flotante(io.leer_linea("Aporte mensual: "));
variable meses = entero(io.leer_linea("Meses (1..120): "));
si capital < 0.0 o aporte < 0.0 o meses < 1 o meses > 120 { lanzar "Parámetros fuera de rango"; }
variable futuro = saldo_futuro(capital, aporte, 0.01, meses);
mostrar("Saldo estimado: " + futuro);
mostrar("Aporte mínimo no negativo: " + math.maximo(aporte, 0.0));
