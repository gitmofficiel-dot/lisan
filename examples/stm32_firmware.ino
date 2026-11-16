// ==================================================================
// تم توليد كود المتحكمات تلقائياً بواسطة لغة لسان (Lisan)
// اللوحة المستهدفة: متحكمات STM32 (ARM Cortex-M: F103 / F401 / Nucleo)
#define LISAN_TARGET_STM32
// ==================================================================

#include <Arduino.h>
#if defined(ARDUINO_ARCH_STM32)
#include <HardwareTimer.h>
#endif
#if defined(ARDUINO_ARCH_STM32)
#include <STM32RTC.h>
#endif
#if defined(ARDUINO_ARCH_STM32)
#include <HalCan.h>
#endif

int lsn_645646641630_627644645624634631 = 13;
int lsn_645646641630_pwm_62764464562d631643 = 5;
int lsn_645646641630_62764462d633627633_62764462a64662763863164a = 0;
int lsn_645646641630_dac = 1;
auto lsn_64562464262a = lsn_64562464262a_63962a62762f64a_stm32(1);
auto lsn_64562d648644_adc = lsn_64562d648644_62a64562762b64464a_stm32(lsn_645646641630_62764462d633627633_62764462a64662763863164a);
auto lsn_64562d648644_dac = lsn_64562d648644_63164264564a_stm32(lsn_645646641630_dac);
auto lsn_643627646 = lsn_646627642644_can_stm32(500);
auto lsn_633627639629 = lsn_633627639629_62764464864262a_62764462d64264a64264a_rtc();
auto lsn_62d641638_627644637627642629 = lsn_648636639_62a64864164a631_637627642629_stm32();

void setup() {
  Serial.begin(115200);
  Serial.println(String("=== بدء تشغيل نظام الأتمتة الصناعية STM32 ARM Cortex-M ==="));
  pinMode(lsn_645646641630_627644645624634631, OUTPUT);
}

void loop() {
  digitalWrite(lsn_645646641630_627644645624634631, HIGH);
  delay(250);
  digitalWrite(lsn_645646641630_627644645624634631, LOW);
  delay(250);
}

