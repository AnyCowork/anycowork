import { Key, WebDriver, By, until } from 'selenium-webdriver';
import { createTauriDriver, cleanupDriver, startNewChat, sendMessageAndWait, E2E_CONFIG } from './setup';

// Extend config for scenarios
const SCENARIO_TIMEOUT = 120000; // 2 minutes per scenario

describe('Daily Usage Scenarios E2E', () => {
    let driver: WebDriver;

    beforeAll(async () => {
        driver = await createTauriDriver();
        // Assuming default agent exists or created by previous tests. 
        try {
            await startNewChat(driver);
        } catch {
            // If fail, likely no agent. This test suite assumes running after agent_modes.
        }
    }, 60000);

    afterAll(async () => {
        await cleanupDriver(driver);
    });

    // Helper to switch mode
    async function setMode(targetMode: 'Planning' | 'Fast') {
        const otherMode = targetMode === 'Planning' ? 'Fast' : 'Planning';
        try {
            // Look for button showing ONLY the OTHER mode (meaning we need to toggle)
            // But wait, the button shows CURRENT mode.
            // If we want Planning, we look for "Fast" text (meaning current is Fast) and click it?
            // User code: 
            // setExecutionMode(executionMode === "planning" ? "fast" : "planning")
            // label: <span className="capitalize">{executionMode}</span>
            // So if button says "Planning", mode IS Planning.

            // If we want targetMode, check if button says targetMode.
            try {
                await driver.findElement(By.xpath(`//button[.//span[contains(text(), "${targetMode}")]]`));
                // Already in target mode
                return;
            } catch {
                // Not in target mode, find the other one and click
                const btn = await driver.findElement(By.xpath(`//button[.//span[contains(text(), "${otherMode}")]]`));
                await btn.click();
            }
        } catch (e) {
            console.error(`Failed to switch to ${targetMode}`, e);
        }
    }

    // Scenario 1: File Management
    test('Scenario: File Management (Create/Read)', async () => {
        await startNewChat(driver);
        await setMode('Planning');

        // 1. Create File
        const filename = `test_scenario_${Date.now()}.txt`;
        await sendMessageAndWait(driver, `Create a file named ${filename} with content "Hello World"`);

        // Verify Success UI
        const planItems = await driver.findElements(By.css('svg.text-green-500'));
        if (planItems.length === 0) throw new Error("File creation task did not complete successfully");

        // 2. Read File (Fast Mode)
        await setMode('Fast');

        await sendMessageAndWait(driver, `Read content of ${filename}`);

        // Verify content
        const lastMsg = (await driver.findElements(By.css('[data-testid="assistant-message"]'))).pop();
        const text = await lastMsg?.getText();
        if (!text?.includes("Hello World")) throw new Error(`Expected "Hello World" in response, got: ${text}`);

    }, SCENARIO_TIMEOUT);

    // Scenario 2: System Query
    test('Scenario: System Query (Fast Mode)', async () => {
        await startNewChat(driver);
        await setMode('Fast');

        await sendMessageAndWait(driver, "What is the current working directory?");

        const lastMsg = (await driver.findElements(By.css('[data-testid="assistant-message"]'))).pop();
        const text = await lastMsg?.getText();
        if (!text?.includes("/")) throw new Error(`Expected a path in response, got: ${text}`);
    }, SCENARIO_TIMEOUT);

    // Scenario 3: Code Generation & Plan
    test('Scenario: Code Gen (Planning)', async () => {
        await startNewChat(driver);
        await setMode('Planning');

        await sendMessageAndWait(driver, "Write a Python script to calculate Fibonacci sequence and save it to fib.py");

        const sidebar = await driver.findElement(By.xpath('//h3[contains(text(), "AGENT PLAN")]'));
        if (!(await sidebar.isDisplayed())) throw new Error("Plan sidebar not visible");

        await driver.wait(until.elementLocated(By.css('svg.text-green-500')), 30000);

    }, SCENARIO_TIMEOUT);

});
