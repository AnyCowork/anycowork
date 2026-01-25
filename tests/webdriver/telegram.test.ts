/**
 * End-to-End Telegram Configuration Tests
 * Tests Telegram bot configuration, save, and persistence workflows
 */

import { WebDriver, By, until } from 'selenium-webdriver';
import { describe, beforeEach, afterEach, test, expect } from '@jest/globals';
import {
    E2E_CONFIG,
    createTauriDriver,
    cleanupDriver,
    takeScreenshot,
} from './setup';

describe('Telegram Configuration E2E Tests', () => {
    let driver: WebDriver;

    beforeEach(async () => {
        driver = await createTauriDriver();
        // Wait for app to load
        await driver.sleep(2000);
    }, 30000);

    afterEach(async () => {
        await cleanupDriver(driver);
    });

    test('E2E-TELEGRAM-001: Save Telegram bot configuration', async () => {
        try {
            // Navigate to settings page
            const settingsLink = await driver.findElement(By.css('a[href="/settings"]'));
            await settingsLink.click();
            await driver.wait(until.urlContains('/settings'), E2E_CONFIG.TIMEOUT.MEDIUM);

            // Click on Messaging tab
            const messagingTab = await driver.findElement(By.css('button[value="messaging"]'));
            await messagingTab.click();

            // Wait for tab content to load
            await driver.sleep(500);

            // Find bot token input
            const tokenInput = await driver.findElement(By.css('input#telegram_bot_token'));
            await tokenInput.clear();
            await tokenInput.sendKeys('1234567890:ABCdefGHIjklMNOpqrsTUVwxyz-TEST');

            // Enable Telegram switch
            const telegramSwitch = await driver.findElement(By.css('button[role="switch"]'));
            const isEnabled = await telegramSwitch.getAttribute('data-state');
            if (isEnabled !== 'checked') {
                await telegramSwitch.click();
            }

            // Click save button
            const saveButton = await driver.findElement(By.xpath('//button[contains(text(), "Save Configuration")]'));
            await saveButton.click();

            // Wait for success toast
            await driver.wait(
                until.elementLocated(By.xpath('//*[contains(text(), "successfully")]')),
                E2E_CONFIG.TIMEOUT.MEDIUM
            );

            // Verify configuration persisted - reload page
            await driver.navigate().refresh();
            await driver.sleep(1000);

            // Click messaging tab again
            const messagingTabAgain = await driver.findElement(By.css('button[value="messaging"]'));
            await messagingTabAgain.click();
            await driver.sleep(500);

            // Verify token is still there
            const tokenInputAfterReload = await driver.findElement(By.css('input#telegram_bot_token'));
            const savedToken = await tokenInputAfterReload.getAttribute('value');
            expect(savedToken).toBe('1234567890:ABCdefGHIjklMNOpqrsTUVwxyz-TEST');

            // Verify switch is still enabled
            const switchAfterReload = await driver.findElement(By.css('button[role="switch"]'));
            const isStillEnabled = await switchAfterReload.getAttribute('data-state');
            expect(isStillEnabled).toBe('checked');

        } catch (error) {
            await takeScreenshot(driver, 'E2E-TELEGRAM-001-failure');
            throw error;
        }
    }, E2E_CONFIG.TIMEOUT.LONG);

    test('E2E-TELEGRAM-002: Disable Telegram bot configuration', async () => {
        try {
            // Navigate to settings
            const settingsLink = await driver.findElement(By.css('a[href="/settings"]'));
            await settingsLink.click();
            await driver.wait(until.urlContains('/settings'), E2E_CONFIG.TIMEOUT.MEDIUM);

            // Click messaging tab
            const messagingTab = await driver.findElement(By.css('button[value="messaging"]'));
            await messagingTab.click();
            await driver.sleep(500);

            // Disable switch if enabled
            const telegramSwitch = await driver.findElement(By.css('button[role="switch"]'));
            const isEnabled = await telegramSwitch.getAttribute('data-state');
            if (isEnabled === 'checked') {
                await telegramSwitch.click();
            }

            // Save
            const saveButton = await driver.findElement(By.xpath('//button[contains(text(), "Save Configuration")]'));
            await saveButton.click();

            // Wait for success
            await driver.wait(
                until.elementLocated(By.xpath('//*[contains(text(), "successfully")]')),
                E2E_CONFIG.TIMEOUT.MEDIUM
            );

        } catch (error) {
            await takeScreenshot(driver, 'E2E-TELEGRAM-002-failure');
            throw error;
        }
    }, E2E_CONFIG.TIMEOUT.LONG);
});
